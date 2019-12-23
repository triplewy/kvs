extern crate failure;

/// Custom errors for KvStore
#[derive(Debug, Fail)]
pub enum KvStoreError {
    /// SerdeError occurs during serialization and deserialization of data
    #[fail(display = "SerdeError: {}", error)]
    SerdeError {
        /// serde error
        error: serde_json::Error,
    },
    /// IoError occurs during File IO
    #[fail(display = "IoError: {}", error)]
    IoError {
        /// io error
        error: std::io::Error,
    },
    /// ParseIntError occurs during parse string to int
    #[fail(display = "ParseIntError: {}", error)]
    ParseIntError {
        /// parseInt error
        error: std::num::ParseIntError,
    },
    /// KeyNotFoundError occurs when a key is not found in KvStore index
    #[fail(display = "Key not found")]
    KeyNotFoundError {},
}

impl From<serde_json::Error> for KvStoreError {
    fn from(error: serde_json::Error) -> Self {
        KvStoreError::SerdeError { error }
    }
}

impl From<std::io::Error> for KvStoreError {
    fn from(error: std::io::Error) -> Self {
        KvStoreError::IoError { error }
    }
}

impl From<std::num::ParseIntError> for KvStoreError {
    fn from(error: std::num::ParseIntError) -> Self {
        KvStoreError::ParseIntError { error }
    }
}
