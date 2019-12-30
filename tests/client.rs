use kvs::thread_pool::*;
use kvs::{KvStore, KvsClient, KvsEngine, KvsServer, Result};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{sync, thread, time};

use num_cpus;
use tempfile::TempDir;

// Test client performing multiple commands
#[test]
fn test_client() -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let engine = "kvs";
    let temp_dir = TempDir::new().unwrap();
    let server = KvsServer::new(
        socket,
        engine,
        KvStore::open(temp_dir.path()).expect("Could not open KvStore"),
        SharedQueueThreadPool::new((num_cpus::get() * 2) as u32)
            .expect("Could not create thread pool"),
    )
    .expect("Could not create server");
    thread::spawn(move || {
        server.start().expect("server stopped");
    });
    thread::sleep(time::Duration::from_secs(2));
    let mut client = KvsClient::new(socket).expect("Could not create client");
    client
        .set(format!("key{}", 0), format!("value{}", 0))
        .expect("Could not set item");
    client = KvsClient::new(socket).expect("Could not create client");
    assert_eq!(
        client.get(format!("key{}", 0)).expect("Could not get"),
        Some(format!("value{}", 0))
    );
    Ok(())
}
