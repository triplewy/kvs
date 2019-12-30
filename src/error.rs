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
    /// Utf8Error occurs when converting bytes to string
    #[fail(display = "UTF8Error: {}", error)]
    Utf8Error {
        /// utf8 error
        error: std::str::Utf8Error,
    },
    /// SledError occurs when interacting with Sled embedded db
    #[fail(display = "SledError: {}", error)]
    SledError {
        /// sled error
        error: sled::Error,
    },
    /// AddrParseError occurs when parsing a string to IpAddr
    #[fail(display = "AddrParseErrror: {}", error)]
    AddrParseError {
        /// addr parse error
        error: std::net::AddrParseError,
    },
    /// KeyNotFoundError occurs when a key is not found in KvStore index
    #[fail(display = "Key not found")]
    KeyNotFoundError {},
    /// ServerError is error from server in response to client request
    #[fail(display = "ServerError: {}", error)]
    ServerError {
        /// server error
        error: String,
    },
    /// RayonError is error from rayon lib
    #[fail(display = "RayonError: {}", error)]
    RayonError {
        /// rayon error
        error: rayon_core::ThreadPoolBuildError,
    },
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

impl From<std::str::Utf8Error> for KvStoreError {
    fn from(error: std::str::Utf8Error) -> Self {
        KvStoreError::Utf8Error { error }
    }
}

impl From<sled::Error> for KvStoreError {
    fn from(error: sled::Error) -> Self {
        KvStoreError::SledError { error }
    }
}

impl From<std::net::AddrParseError> for KvStoreError {
    fn from(error: std::net::AddrParseError) -> Self {
        KvStoreError::AddrParseError { error }
    }
}

impl From<rayon_core::ThreadPoolBuildError> for KvStoreError {
    fn from(error: rayon_core::ThreadPoolBuildError) -> Self {
        KvStoreError::RayonError { error }
    }
}
