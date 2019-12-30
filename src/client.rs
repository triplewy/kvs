use crate::error::KvStoreError;
use crate::kv::Result;
use crate::network::{ClientRequest, ClientRequestType, Response};

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
        let resp: Response = serde_json::from_reader(&mut self.stream)?;
        if resp.error != "" {
            return Err(KvStoreError::ServerError { error: resp.error });
        }
        Ok(resp.value)
    }
    /// get sends a get request to the server
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = ClientRequest {
            command_type: ClientRequestType::Get,
            key: key.to_owned(),
            value: "".to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &req)?;
        let resp: Response = serde_json::from_reader(&mut self.stream)?;
        if resp.error != "" {
            return Err(KvStoreError::ServerError { error: resp.error });
        }
        if resp.value == "".to_owned() {
            return Ok(None);
        }
        Ok(Some(resp.value))
    }
    /// remove sends a remove request to the server
    pub fn remove(&mut self, key: String) -> Result<String> {
        let req = ClientRequest {
            command_type: ClientRequestType::Rm,
            key: key.to_owned(),
            value: "".to_owned(),
        };
        serde_json::to_writer(&mut self.stream, &req)?;
        let resp: Response = serde_json::from_reader(&mut self.stream)?;
        if resp.error != "" {
            return Err(KvStoreError::ServerError { error: resp.error });
        }
        Ok(resp.value)
    }
}
