#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};

use kvs::thread_pool::*;
use kvs::{KvStore, KvsClient, KvsEngine, KvsServer, SledKvsEngine};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{process, sync, thread, time};

use assert_cmd::prelude::*;
use num_cpus;
use tempfile::TempDir;

fn write_queued_kvstore(c: &mut Criterion) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let mut num_threads: Vec<u32> = vec![1];
    for i in (2..(num_cpus::get() * 2 + 1)).step_by(2) {
        num_threads.push(i as u32);
    }
    let mut group = c.benchmark_group("write_queued_kvstore");

    for threads in num_threads.iter() {
        let (sender, receiver) = sync::mpsc::sync_channel(0);
        let temp_dir = TempDir::new().unwrap();
        let mut child = process::Command::cargo_bin("kvs-server")
            .unwrap()
            .args(&["--threads", &threads.to_string()])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        let handle = thread::spawn(move || {
            let _ = receiver.recv(); // wait for main thread to finish
            child.kill().expect("server exited before killed");
        });
        thread::sleep(time::Duration::from_secs(1));
        group.bench_function(BenchmarkId::from_parameter(threads), |b| {
            b.iter(|| {
                let barrier = sync::Arc::new(sync::Barrier::new(11));
                for i in 0..10 {
                    let barrier = barrier.clone();
                    thread::spawn(move || {
                        let mut client = KvsClient::new(socket).expect("Could not create client");
                        client
                            .set(format!("key{}", i), format!("value{}", i))
                            .expect("Could not set item");
                        barrier.wait();
                    });
                }
                barrier.wait();
            });
        });
        sender.send(()).unwrap();
        handle.join().unwrap();
    }
    group.finish();
}

fn read_queued_kvstore(c: &mut Criterion) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let mut num_threads: Vec<u32> = vec![1];
    for i in (2..(num_cpus::get() * 2 + 1)).step_by(2) {
        num_threads.push(i as u32);
    }
    let mut group = c.benchmark_group("write_queued_kvstore");
    for threads in num_threads.iter() {
        let (sender, receiver) = sync::mpsc::sync_channel(0);
        let temp_dir = TempDir::new().unwrap();
        let mut child = process::Command::cargo_bin("kvs-server")
            .unwrap()
            .args(&["--threads", &threads.to_string()])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        let handle = thread::spawn(move || {
            let _ = receiver.recv(); // wait for main thread to finish
            child.kill().expect("server exited before killed");
        });
        thread::sleep(time::Duration::from_secs(1));
        for i in 0..10 {
            let mut client = KvsClient::new(socket).expect("Could not create client");
            client
                .set(format!("key{}", i), format!("value{}", i))
                .expect("Could not set item");
        }
        group.bench_function(BenchmarkId::from_parameter(threads), |b| {
            b.iter(|| {
                let barrier = sync::Arc::new(sync::Barrier::new(11));
                for i in 0..10 {
                    let barrier = barrier.clone();
                    thread::spawn(move || {
                        let mut client = KvsClient::new(socket).expect("Could not create client");
                        assert_eq!(
                            client.get(format!("key{}", i)).expect("Could not set item"),
                            Some(format!("value{}", i))
                        );
                        barrier.wait();
                    });
                }
                barrier.wait();
            });
        });
        sender.send(()).unwrap();
        handle.join().unwrap();
    }
    group.finish();
}

fn write_rayon_kvstore(c: &mut Criterion) {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);
    let mut num_threads: Vec<u32> = vec![1];
    for i in (2..(num_cpus::get() * 2 + 1)).step_by(2) {
        num_threads.push(i as u32);
    }
    let mut group = c.benchmark_group("write_rayon_kvstore");

    for threads in num_threads.iter() {
        let (sender, receiver) = sync::mpsc::sync_channel(0);
        let temp_dir = TempDir::new().unwrap();
        let mut child = process::Command::cargo_bin("kvs-server")
            .unwrap()
            .args(&["--threads", &threads.to_string(), "--pool", "rayon"])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        let handle = thread::spawn(move || {
            let _ = receiver.recv(); // wait for main thread to finish
            child.kill().expect("server exited before killed");
        });
        thread::sleep(time::Duration::from_secs(1));
        group.bench_function(BenchmarkId::from_parameter(threads), |b| {
            b.iter(|| {
                let barrier = sync::Arc::new(sync::Barrier::new(11));
                for i in 0..10 {
                    let barrier = barrier.clone();
                    thread::spawn(move || {
                        let mut client = KvsClient::new(socket).expect("Could not create client");
                        client
                            .set(format!("key{}", i), format!("value{}", i))
                            .expect("Could not set item");
                        barrier.wait();
                    });
                }
                barrier.wait();
            });
        });
        sender.send(()).unwrap();
        handle.join().unwrap();
    }
    group.finish();
}

criterion_group!(benches, write_rayon_kvstore);
criterion_main!(benches);
