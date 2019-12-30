use crate::{KvStoreError, Result};

use sled::Db;
use std::path::Path;
use std::str::from_utf8;

/// KvsEngine is a trait for plug-in database engines to implement
pub trait KvsEngine: Clone + Send + 'static {
    /// Set the value of a string key to a string.
    /// Return an error if the value is not written successfully.
    fn set(&self, key: String, value: String) -> Result<()>;
    /// Get the string value of a string key. If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    fn get(&self, key: String) -> Result<Option<String>>;
    /// Remove a given string key.
    /// Return an error if the key does not exit or value is not read successfully.
    fn remove(&self, key: String) -> Result<()>;
}

/// SledKvsEngine implements the KvsEngine
#[derive(Clone)]
pub struct SledKvsEngine {
    db: Db,
}

impl SledKvsEngine {
    /// open calls sled's open and returns the db
    pub fn open(path: &Path) -> Result<Self> {
        let db = Db::open(path)?;
        Ok(SledKvsEngine { db })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        match self.db.get(key.as_bytes())? {
            Some(v) => {
                let vec = v.as_ref();
                let s = from_utf8(vec)?;
                Ok(Some(s.to_owned()))
            }
            None => Ok(None),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        let res = self.db.remove(key)?;
        match res {
            Some(_) => {
                self.db.flush()?;
                Ok(())
            }
            None => Err(KvStoreError::KeyNotFoundError {}),
        }
    }
}
