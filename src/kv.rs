//! In-memory kv store
use std::collections::HashMap;

/// KvStore is an in-memory database that maps strings to string
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// Creates a new instance of KvStore
    /// ```rust
    /// use kvs::KvStore;
    /// fn main() {
    ///     let store = KvStore::new();
    /// }
    /// ```
    pub fn new() -> KvStore {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Writes a key, value pair to KvStore
    /// ```rust
    /// # use kvs::KvStore;
    /// fn main() {
    ///     let mut store = KvStore::new();
    ///     store.set(String::from("key"), String::from("value"));
    /// }
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// Reads the value for a key
    /// ```rust
    /// # use kvs::KvStore;
    /// fn main() {
    ///     let mut store = KvStore::new();
    ///     store.set(String::from("key"), String::from("value"));
    ///     match store.get(String::from("key")) {
    ///         Some(v) => assert_eq!(v, String::from("value")),
    ///         None => panic!(),
    ///     }
    /// }
    /// ```
    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).cloned()
    }

    /// Removes a key from the KvStore. Succeeds regardless if key exists in KvStore.
    /// ```rust
    /// # use kvs::KvStore;
    /// fn main() {
    ///     let mut store = KvStore::new();
    ///     store.set(String::from("key"), String::from("value"));
    ///     store.remove(String::from("key"));
    ///     assert_eq!(store.get(String::from("key")), None);
    /// }
    /// ```
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
