use crate::engine::KvsEngine;
use crate::kv::Result;
use crate::network::{ClientRequest, ClientRequestType, Response};
use crate::thread_pool::*;

use serde::de::Deserialize;
use slog::Drain;
use std::env;
use std::net::{SocketAddr, TcpListener, TcpStream};

/// KvsServer is a TCP server that handles client cmduests to the underlying KvStore
pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    socket: SocketAddr,
    log: slog::Logger,
    db: E,
    pool: P,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    /// Instantiates new KvsServer with log and db engine
    pub fn new(socket: SocketAddr, engine_name: &str, engine: E, pool: P) -> Result<Self> {
        let decorator = slog_term::TermDecorator::new().stderr().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());

        info!(log, "{}", env!("CARGO_PKG_VERSION"));
        info!(log, "{}", socket);
        info!(log, "{}", engine_name);

        Ok(KvsServer {
            socket,
            log,
            db: engine,
            pool,
        })
    }

    /// Starts KvsServer and listens for connections
    pub fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.socket)?;

        for stream in listener.incoming() {
            let db = self.db.clone();
            let log = self.log.clone();
            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = process_cmd(db, stream) {
                        error!(log, "{}", e.to_string());
                    }
                }
                Err(e) => error!(log, "{}", e),
            });
        }
        Ok(())
    }
}

fn process_cmd<E: KvsEngine>(db: E, stream: TcpStream) -> Result<()> {
    let mut de = serde_json::Deserializer::from_reader(&stream);
    let cmd = ClientRequest::deserialize(&mut de)?;
    let mut resp = Response::default();
    match cmd.command_type {
        ClientRequestType::Set => match db.set(cmd.key, cmd.value) {
            Ok(_) => {
                resp.value = "OK".to_owned();
            }
            Err(e) => {
                resp.error = e.to_string();
            }
        },
        ClientRequestType::Rm => match db.remove(cmd.key) {
            Ok(_) => {
                resp.value = "OK".to_owned();
            }
            Err(e) => {
                resp.error = e.to_string();
            }
        },
        ClientRequestType::Get => match db.get(cmd.key) {
            Ok(res) => match res {
                Some(value) => {
                    resp.value = value;
                }
                None => {
                    resp.value = "".to_owned();
                }
            },
            Err(e) => {
                resp.error = e.to_string();
            }
        },
    }
    serde_json::to_writer(stream, &resp)?;
    Ok(())
}
