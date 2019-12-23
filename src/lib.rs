//! A crate for a database that maps String to String
#![deny(missing_docs)]

#[macro_use]
extern crate failure_derive;

mod config;
mod error;
mod kv;

pub use config::Config;
pub use error::KvStoreError;
pub use kv::{KvStore, Result};
