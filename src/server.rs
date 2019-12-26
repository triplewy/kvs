use crate::engine::KvsEngine;
use crate::kv::{KvStore, Result};
use crate::network::{ClientRequest, ClientRequestType, ErrorResponse, ValueResponse};

use serde::de::Deserialize;
use slog::Drain;
use std::env;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener};

/// KvsServer is a TCP server that handles client requests to the underlying KvStore
pub struct KvsServer<E: KvsEngine> {
    socket: SocketAddr,
    log: slog::Logger,
    db: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// Instantiates new KvsServer with log and db engine
    pub fn new(socket: SocketAddr, engine: E, engine_name: &str) -> Result<Self> {
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
        })
    }

    /// Starts KvsServer and listens for connections
    pub fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.socket)?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    info!(self.log, "new client!");
                    let mut de = serde_json::Deserializer::from_reader(&mut stream);
                    let req = ClientRequest::deserialize(&mut de)?;
                    info!(self.log, "req: {:?}", req);
                    match req.command_type {
                        ClientRequestType::Set => match self.db.set(req.key, req.value) {
                            Ok(_) => {
                                stream.write(&[0])?;
                                let resp = ValueResponse {
                                    value: "OK".to_owned(),
                                };
                                serde_json::to_writer(stream, &resp)?
                            }
                            Err(e) => {
                                error!(self.log, "{}", e.to_string());
                                stream.write(&[1])?;
                                let resp = ErrorResponse {
                                    error: e.to_string(),
                                };
                                serde_json::to_writer(stream, &resp)?
                            }
                        },
                        ClientRequestType::Rm => match self.db.remove(req.key) {
                            Ok(_) => {
                                stream.write(&[0])?;
                                let resp = ValueResponse {
                                    value: "OK".to_owned(),
                                };
                                serde_json::to_writer(stream, &resp)?
                            }
                            Err(e) => {
                                error!(self.log, "{}", e.to_string());
                                stream.write(&[1])?;
                                let resp = ErrorResponse {
                                    error: e.to_string(),
                                };
                                serde_json::to_writer(stream, &resp)?
                            }
                        },
                        ClientRequestType::Get => match self.db.get(req.key) {
                            Ok(res) => {
                                stream.write(&[0])?;
                                match res {
                                    Some(value) => {
                                        let resp = ValueResponse { value };
                                        serde_json::to_writer(stream, &resp)?
                                    }
                                    None => {
                                        let resp = ValueResponse {
                                            value: "".to_owned(),
                                        };
                                        serde_json::to_writer(stream, &resp)?
                                    }
                                }
                            }
                            Err(e) => {
                                error!(self.log, "{}", e.to_string());
                                stream.write(&[1])?;
                                let resp = ErrorResponse {
                                    error: e.to_string(),
                                };
                                serde_json::to_writer(stream, &resp)?
                            }
                        },
                    }
                }
                Err(e) => error!(self.log, "{}", e),
            }
        }
        Ok(())
    }
}
