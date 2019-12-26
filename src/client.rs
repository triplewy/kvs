use crate::error::KvStoreError;
use crate::kv::Result;
use crate::network::{ClientRequest, ClientRequestType, ErrorResponse, ValueResponse};

use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};

/// KvsClient sends requests to KvsServer
pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    /// new establishes a TcpStream and instantiates client
    pub fn new(socket: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(socket)?;
        Ok(KvsClient { stream })
    }

    /// set sends a set request to the server
    pub fn set(&mut self, key: String, value: String) -> Result<String> {
        let req = ClientRequest {
            command_type: ClientRequestType::Set,
            key: key.to_owned(),
            value: value.to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &req)?;
        let mut resp = [0; 1];
        self.stream.read(&mut resp)?;
        if resp == [1; 1] {
            let err: ErrorResponse = serde_json::from_reader(&mut self.stream)?;
            return Err(KvStoreError::ServerError { error: err.error });
        }
        let value: ValueResponse = serde_json::from_reader(&mut self.stream)?;
        Ok(value.value)
    }
    /// get sends a get request to the server
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = ClientRequest {
            command_type: ClientRequestType::Get,
            key: key.to_owned(),
            value: "".to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &req)?;
        let mut resp = [0; 1];
        self.stream.read(&mut resp)?;
        if resp == [1; 1] {
            let err: ErrorResponse = serde_json::from_reader(&mut self.stream)?;
            return Err(KvStoreError::ServerError { error: err.error });
        }
        let value: ValueResponse = serde_json::from_reader(&mut self.stream)?;
        if value.value == String::from("") {
            return Ok(None);
        }
        Ok(Some(value.value))
    }
    /// remove sends a remove request to the server
    pub fn remove(&mut self, key: String) -> Result<String> {
        let req = ClientRequest {
            command_type: ClientRequestType::Rm,
            key: key.to_owned(),
            value: "".to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &req)?;
        let mut resp = [0; 1];
        self.stream.read(&mut resp)?;
        if resp == [1; 1] {
            let err: ErrorResponse = serde_json::from_reader(&mut self.stream)?;
            return Err(KvStoreError::ServerError { error: err.error });
        }
        let value: ValueResponse = serde_json::from_reader(&mut self.stream)?;
        Ok(value.value)
    }
}
