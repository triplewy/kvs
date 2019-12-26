#[macro_use]
extern crate clap;

use clap::App;
use kvs::{KvStore, KvsEngine, KvsServer, Result, SledKvsEngine};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{env, fs, process};

fn main() -> Result<()> {
    let yaml = load_yaml!("server.yml");
    let matches = App::from_yaml(yaml)
        .name(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    let socket = match matches.value_of("addr") {
        Some(v) => v.parse()?,
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000),
    };

    let curr_dir = env::current_dir()?;
    let path = curr_dir.join("engine");
    let mut engine = "";
    fs::create_dir_all(&path)?;
    if path.join("kvs").exists() {
        engine = "kvs";
    } else if path.join("sled").exists() {
        engine = "sled";
    }

    match matches.value_of("engine") {
        Some(v) => {
            if engine == "" {
                engine = v;
            } else if engine != v {
                eprintln!("Selected engine does not match previous data");
                process::exit(1);
            }
        }
        None => {
            if engine == "" {
                engine = "kvs";
            }
        }
    }

    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path.join(engine))?;

    match engine {
        "kvs" => run(socket, KvStore::open(&curr_dir)?, &engine)?,
        "sled" => run(socket, SledKvsEngine::open(&curr_dir)?, &engine)?,
        _ => unreachable!(),
    };

    Ok(())
}

fn run<E: KvsEngine>(socket: SocketAddr, engine: E, engine_name: &str) -> Result<()> {
    let mut server = KvsServer::new(socket, engine, engine_name)?;
    server.start()
}
