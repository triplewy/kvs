#[macro_use]
extern crate clap;

use clap::App;
use kvs::thread_pool::*;
use kvs::{KvStore, KvsEngine, KvsServer, Result, SledKvsEngine};
use num_cpus;
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

    let num_threads = match matches.value_of("threads") {
        Some(v) => v.parse::<u32>()?,
        None => num_cpus::get() as u32,
    };
    println!("num_threads: {}", num_threads);

    let pool = match matches.value_of("pool") {
        Some(v) => v,
        None => "crossbeam",
    };

    if pool == "crossbeam" {
        if engine == "kvs" {
            run(
                socket,
                &engine,
                KvStore::open(&curr_dir)?,
                SharedQueueThreadPool::new(num_threads)?,
            )?;
        } else if engine == "sled" {
            run(
                socket,
                &engine,
                SledKvsEngine::open(&curr_dir)?,
                SharedQueueThreadPool::new(num_threads)?,
            )?;
        } else {
            unreachable!()
        }
    } else if pool == "rayon" {
        if engine == "kvs" {
            run(
                socket,
                &engine,
                KvStore::open(&curr_dir)?,
                RayonThreadPool::new(num_threads)?,
            )?;
        } else if engine == "sled" {
            run(
                socket,
                &engine,
                SledKvsEngine::open(&curr_dir)?,
                RayonThreadPool::new(num_threads)?,
            )?;
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }

    Ok(())
}

fn run<E: KvsEngine, P: ThreadPool>(
    socket: SocketAddr,
    engine_name: &str,
    engine: E,
    pool: P,
) -> Result<()> {
    let server = KvsServer::new(socket, engine_name, engine, pool)?;
    server.start()
}
