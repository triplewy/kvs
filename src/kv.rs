//! In-memory kv store

use crate::config::Config;
use crate::engine::KvsEngine;
use crate::error::KvStoreError;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir_all, remove_file, rename, File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use tempfile::{Builder, NamedTempFile};

/// Result is alias for std::result::Result that defaults KvStoreError
pub type Result<T> = std::result::Result<T, KvStoreError>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum CommandType {
    Set,
    Rm,
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    cmd: CommandType,
    key: String,
    value: String,
}

#[derive(Debug, Clone)]
struct FilePointer {
    path: PathBuf,
    offset: u64,
}

/// KvStore is an in-memory database that maps strings to string
#[derive(Clone)]
pub struct KvStore {
    map: Arc<RwLock<HashMap<String, FilePointer>>>,
    writer: Arc<Mutex<BufWriter<File>>>,
    id: Arc<Mutex<u16>>,
    path: PathBuf,
    config: Config,
}

impl KvsEngine for KvStore {
    /// Writes a key, value pair to KvStore. Can potentially cause compaction which will block method until completed
    /// ```rust
    /// # use kvs::{KvStore, Result, KvsEngine};
    /// # use std::env;
    /// # fn main() -> Result<()> {
    /// let curr_dir = env::current_dir().unwrap();
    /// let mut store = KvStore::open(curr_dir.as_path()).expect("Failed to open KvStore");
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// # Ok(())
    /// # }
    /// ```
    fn set(&self, key: String, value: String) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        let mut id = self.id.lock().unwrap();
        let mut offset = writer.seek(SeekFrom::Current(0))?;
        // If current file is above filesize limit, create new log file
        if offset > self.config.filesize_limit {
            // Compact files if current id is divisible by compaction_thresh
            if *id > 0 && *id % self.config.compaction_thresh * 2 == 0 {
                let max_id = *id;
                let store = self.clone();
                thread::spawn(move || {
                    let temp_file = Builder::new()
                        .append(true)
                        .tempfile()
                        .expect("Could not create tempfile");
                    let (temp_map, immutable_ids) = store
                        .compact(&temp_file, max_id)
                        .expect("Could not compact files");
                    store
                        .merge(temp_file.path(), temp_map, immutable_ids, max_id + 1)
                        .expect("Could not merge files");
                });
            }
            *id += 2;
            let f = OpenOptions::new()
                .append(true)
                .create(true)
                .open(get_log_path(&self.path, *id))?;
            *writer = BufWriter::new(f);
            offset = 0;
        }
        let mut map = self.map.write().unwrap();
        // Write new entry to log
        let cmd = Command {
            cmd: CommandType::Set,
            key: key.clone(),
            value: value,
        };
        serde_json::to_writer(&mut *writer, &cmd)?;
        writer.flush()?;
        let path = get_log_path(&self.path, *id);
        map.insert(
            key,
            FilePointer {
                path: path,
                offset: offset,
            },
        );
        Ok(())
    }

    /// Reads a value for a key. If key is not found, will return Ok(None)
    /// ```rust
    /// # use kvs::{KvStore, Result, KvsEngine};
    /// # use std::env;
    /// # fn main() -> Result<()> {
    /// let curr_dir = env::current_dir().unwrap();
    /// let mut store = KvStore::open(curr_dir.as_path())?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// assert_eq!(Some("value1".to_owned()), store.get("key1".to_owned())?);
    /// assert_eq!(None, store.get("key2".to_owned())?);
    /// # Ok(())
    /// # }
    /// ```
    fn get(&self, key: String) -> Result<Option<String>> {
        let map = self.map.read().unwrap();
        match map.get(&key) {
            Some(fp) => {
                let f = File::open(&fp.path)?;
                let mut reader = BufReader::new(f);
                reader.seek(SeekFrom::Start(fp.offset))?;
                let mut stream =
                    serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
                if let Some(res) = stream.next() {
                    let cmd: Command = res?;
                    return Ok(Some(cmd.value));
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Removes a key from the KvStore. Succeeds regardless if key exists in KvStore.
    /// ```rust
    /// # use kvs::{KvStore, Result, KvsEngine};
    /// # use std::env;
    /// # fn main() -> Result<()> {
    /// let curr_dir = env::current_dir().unwrap();
    /// let mut store = KvStore::open(curr_dir.as_path())?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// store.remove("key1".to_owned())?;
    /// assert_eq!(None, store.get("key1".to_owned())?);
    /// # Ok(())
    /// # }
    /// ```
    fn remove(&self, key: String) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let mut writer = self.writer.lock().unwrap();
        match map.get(&key) {
            Some(_) => {
                let cmd = Command {
                    cmd: CommandType::Rm,
                    key: key.clone(),
                    value: String::default(),
                };
                serde_json::to_writer(&mut *writer, &cmd)?;
                writer.flush()?;
                map.remove(&key);
                Ok(())
            }
            None => Err(KvStoreError::KeyNotFoundError {}),
        }
    }
}

impl KvStore {
    /// Open loads all log data inside the given path and assigns a new writer to write entries to
    /// ```
    /// use kvs::KvStore;
    /// use std::env;
    /// fn main() {
    ///     let curr_dir = env::current_dir().unwrap();
    ///     let store = KvStore::open(curr_dir.as_path()).expect("Failed to open KvStore");
    /// }
    pub fn open(path: &Path) -> Result<KvStore> {
        let dir = path.join("logs");
        create_dir_all(&dir)?;
        let (map, last_id) = load(&dir)?;
        let f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(get_log_path(&dir, last_id))?;
        let mut writer = BufWriter::new(f);
        writer.seek(SeekFrom::End(0))?;
        Ok(KvStore {
            map: Arc::new(RwLock::new(map)),
            writer: Arc::new(Mutex::new(writer)),
            id: Arc::new(Mutex::new(last_id)),
            path: dir,
            config: Config::default(),
        })
    }

    // Compaction: Populate tempfile and tempmap. Only requires immutable ref to self
    fn compact(
        &self,
        temp_file: &NamedTempFile,
        max_id: u16,
    ) -> Result<(HashMap<String, FilePointer>, HashSet<PathBuf>)> {
        let mut writer = BufWriter::new(temp_file);
        let mut temp_map: HashMap<String, FilePointer> = HashMap::new();
        let mut offset = 0u64;
        let mut immutable_ids: HashSet<PathBuf> = HashSet::new();
        let map = self.map.read().unwrap();
        for res in fs::read_dir(&self.path)? {
            let entry = res?;
            let path = entry.path();
            if let Some(id) = get_log_id(&path)? {
                if id <= max_id {
                    let f = File::open(&path)?;
                    let reader = BufReader::new(f);
                    let mut stream =
                        serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
                    let mut read_offset = 0u64;
                    while let Some(res) = stream.next() {
                        let cmd: Command = res?;
                        match cmd.cmd {
                            CommandType::Set => {
                                if let Some(v) = map.get(&cmd.key) {
                                    if v.path == path.clone() && v.offset == read_offset {
                                        serde_json::to_writer(&mut writer, &cmd)?;
                                        temp_map.insert(
                                            cmd.key,
                                            FilePointer {
                                                path: temp_file.path().to_owned(),
                                                offset: offset,
                                            },
                                        );
                                        offset = writer.seek(SeekFrom::Current(0))?;
                                    }
                                }
                            }
                            _ => (),
                        }
                        read_offset = stream.byte_offset() as u64;
                    }
                    immutable_ids.insert(path);
                }
            }
        }
        Ok((temp_map, immutable_ids))
    }
    // Merge: Rename tempfile and update map. Requires mutable ref to self
    fn merge(
        &self,
        old_path: &Path,
        temp_map: HashMap<String, FilePointer>,
        immutable_ids: HashSet<PathBuf>,
        id: u16,
    ) -> Result<()> {
        let new_path = get_log_path(&self.path, id);
        rename(old_path, &new_path)?;
        let mut map = self.map.write().unwrap();
        for (key, value) in &temp_map {
            if let Some(fp) = map.get(key) {
                if let Some(file_id) = get_log_id(&fp.path)? {
                    if file_id > id {
                        continue;
                    }
                }
            }
            map.insert(
                key.to_owned(),
                FilePointer {
                    path: new_path.clone(),
                    offset: value.offset,
                },
            );
        }
        for path in &immutable_ids {
            remove_file(path)?;
        }
        Ok(())
    }
}

fn get_log_path(path: &PathBuf, id: u16) -> PathBuf {
    let mut log_path = path.join(id.to_string());
    log_path.set_extension("log");
    log_path
}

fn get_log_id(path: &PathBuf) -> Result<Option<u16>> {
    if let Some(ext) = path.extension() {
        if *ext == *"log" {
            if let Some(id) = path.file_stem() {
                if let Some(id_str) = id.to_str() {
                    let num_id = id_str.parse::<u16>()?;
                    return Ok(Some(num_id));
                }
            }
        }
    }
    Ok(None)
}

fn load(path: &Path) -> Result<(HashMap<String, FilePointer>, u16)> {
    // Find all log files and sort them in asc order
    let mut ids: Vec<u16> = Vec::new();
    for res in fs::read_dir(path)? {
        let entry = res?;
        let entry_path = entry.path();
        if let Some(id) = get_log_id(&entry_path)? {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    let mut last_id = 0u16;
    if ids.len() > 0 {
        last_id = ids[ids.len() - 1];
    }
    // Read files in order and load into map
    let mut map: HashMap<String, FilePointer> = HashMap::new();
    for id in ids {
        let path_buf = get_log_path(&path.to_owned(), id);
        let f = File::open(&path_buf)?;
        let reader = BufReader::new(f);
        let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
        let mut offset = 0u64;
        while let Some(res) = stream.next() {
            let cmd: Command = res?;
            match cmd.cmd {
                CommandType::Set => {
                    map.insert(
                        cmd.key,
                        FilePointer {
                            path: path_buf.clone(),
                            offset: offset,
                        },
                    );
                }
                CommandType::Rm => {
                    map.remove(&cmd.key);
                }
            }
            offset = stream.byte_offset() as u64;
        }
    }
    Ok((map, last_id))
}
