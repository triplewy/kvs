//! A crate for a database that maps String to String
#![deny(missing_docs)]

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate slog;

mod client;
mod config;
mod engine;
mod error;
mod kv;
mod network;
mod server;
/// thread_pool contains various thread pool implementations
pub mod thread_pool;

pub use client::KvsClient;
pub use config::Config;
pub use engine::{KvsEngine, SledKvsEngine};
pub use error::KvStoreError;
pub use kv::{KvStore, Result};
pub use network::{ClientRequest, ClientRequestType, Response};
pub use server::KvsServer;
