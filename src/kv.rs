//! In-memory kv store

use crate::config::Config;
use crate::error::KvStoreError;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, remove_file, rename, File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
struct FilePointer {
    path: PathBuf,
    offset: u64,
}

/// KvStore is an in-memory database that maps strings to string
pub struct KvStore {
    map: HashMap<String, FilePointer>,
    writer: BufWriter<File>,
    path: PathBuf,
    id: u16,
    immutable_ids: HashSet<PathBuf>,
    config: Config,
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
        let (map, immutable_ids, last_id) = load(path)?;
        let log = get_log_path(&path.to_owned(), last_id);
        let f = OpenOptions::new().append(true).create(true).open(log)?;
        let mut writer = BufWriter::new(f);
        writer.seek(SeekFrom::End(0))?;
        Ok(KvStore {
            map: map,
            writer: writer,
            path: path.to_owned(),
            id: last_id,
            immutable_ids: immutable_ids,
            config: Config::default(),
        })
    }

    /// Writes a key, value pair to KvStore. Can potentially cause compaction which will block method until completed
    /// ```rust
    /// # use kvs::{KvStore, Result};
    /// # use std::env;
    /// # fn main() -> Result<()> {
    /// let curr_dir = env::current_dir().unwrap();
    /// let mut store = KvStore::open(curr_dir.as_path()).expect("Failed to open KvStore");
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut offset = self.writer.seek(SeekFrom::Current(0))?;
        // If current file is above filesize limit, create new log file
        if offset > self.config.filesize_limit {
            self.immutable_ids.insert(get_log_path(&self.path, self.id));
            // Compact files if current id is divisible by compaction_thresh
            if self.id > 0 && self.id % self.config.compaction_thresh == 0 {
                let temp_file = Builder::new().append(true).tempfile()?;
                let (old_path, temp_map) = self.compact(&temp_file)?;
                self.merge(old_path, temp_map)?;
            }
            self.id += 1;
            let f = OpenOptions::new()
                .append(true)
                .create(true)
                .open(get_log_path(&self.path, self.id))?;
            self.writer = BufWriter::new(f);
            offset = 0;
        }
        // Write new entry to log
        let cmd = Command {
            cmd: CommandType::Set,
            key: key.clone(),
            value: value,
        };
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        let path = get_log_path(&self.path, self.id);
        self.map.insert(
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
    /// # use kvs::{KvStore, Result};
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
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.map.get(&key) {
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
    /// # use kvs::{KvStore, Result};
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
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.map.get(&key) {
            Some(_) => {
                let cmd = Command {
                    cmd: CommandType::Rm,
                    key: key.clone(),
                    value: String::default(),
                };
                serde_json::to_writer(&mut self.writer, &cmd)?;
                self.map.remove(&key);
                Ok(())
            }
            None => Err(KvStoreError::KeyNotFoundError {}),
        }
    }

    // Compaction: Populate tempfile and tempmap. Only requires immutable ref to self
    fn compact(
        &self,
        temp_file: &NamedTempFile,
    ) -> Result<(PathBuf, HashMap<String, FilePointer>)> {
        let temp_path = temp_file.path().to_owned();
        let mut writer = BufWriter::new(temp_file);
        let mut temp_map: HashMap<String, FilePointer> = HashMap::new();
        let mut offset = 0u64;
        for path in &self.immutable_ids {
            let f = File::open(&path)?;
            let reader = BufReader::new(f);
            let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
            let mut read_offset = 0u64;
            while let Some(res) = stream.next() {
                let cmd: Command = res?;
                match cmd.cmd {
                    CommandType::Set => {
                        if let Some(v) = self.map.get(&cmd.key) {
                            if v.path == path.clone() && v.offset == read_offset {
                                serde_json::to_writer(&mut writer, &cmd)?;
                                temp_map.insert(
                                    cmd.key,
                                    FilePointer {
                                        path: temp_path.clone(),
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
        }
        Ok((temp_path, temp_map))
    }
    // Merge: Rename tempfile and update map. Requires mutable ref to self
    fn merge(&mut self, old_path: PathBuf, temp_map: HashMap<String, FilePointer>) -> Result<()> {
        self.id += 1;
        let new_path = get_log_path(&self.path, self.id);
        rename(old_path, &new_path)?;
        for (key, value) in &temp_map {
            self.map.insert(
                key.to_owned(),
                FilePointer {
                    path: new_path.clone(),
                    offset: value.offset,
                },
            );
        }
        for path in &self.immutable_ids {
            remove_file(path)?;
        }
        self.immutable_ids = HashSet::new();
        self.immutable_ids.insert(new_path);
        Ok(())
    }
}

fn get_log_path(path: &PathBuf, id: u16) -> PathBuf {
    let mut log_path = path.join(id.to_string());
    log_path.set_extension("log");
    log_path
}

fn load(path: &Path) -> Result<(HashMap<String, FilePointer>, HashSet<PathBuf>, u16)> {
    // Find all log files and sort them in asc order
    let mut ids: Vec<u16> = Vec::new();
    let mut immutable_ids: HashSet<PathBuf> = HashSet::new();
    for res in fs::read_dir(path)? {
        let entry = res?;
        let entry_path = entry.path();
        immutable_ids.insert(entry_path.clone());
        if let Some(ext) = entry_path.extension() {
            if *ext == *"log" {
                if let Some(id) = entry_path.file_stem() {
                    if let Some(id_str) = id.to_str() {
                        let num_id = id_str.parse::<u16>()?;
                        ids.push(num_id)
                    }
                }
            }
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
    Ok((map, immutable_ids, last_id))
}
