use crate::Result;

use crossbeam_channel::{unbounded, Receiver, Sender};
use rayon::prelude::*;
use std::thread;

/// ThreadPool is trait for spawning multiple worker threads to complete jobs
pub trait ThreadPool {
    /// new initializes all the worker threads
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;
    /// spawn moves a job to a worker thread for completion
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

/// NaiveThreadPool is a naive implementation of ThreadPool
pub struct NaiveThreadPool {
    threads: u32,
}

impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> Result<Self> {
        Ok(NaiveThreadPool { threads })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(move || job());
    }
}

/// Shared queue thread pool
pub struct SharedQueueThreadPool {
    sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let (sender, receiver) = unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..threads {
            let rx = TaskReceiver(receiver.clone());
            thread::Builder::new().spawn(move || run_tasks(rx))?;
        }
        Ok(SharedQueueThreadPool { sender })
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender
            .send(job)
            .expect("The thread pool has no thread`");
    }
}

#[derive(Clone)]
struct TaskReceiver(Receiver<Box<dyn FnOnce() + Send + 'static>>);

impl Drop for TaskReceiver {
    fn drop(&mut self) {
        if thread::panicking() {
            let rx = self.clone();
            if let Err(e) = thread::Builder::new().spawn(move || run_tasks(rx)) {
                eprintln!("{}", e.to_string());
            }
        }
    }
}

fn run_tasks(rx: TaskReceiver) {
    loop {
        match rx.0.recv() {
            Ok(job) => job(),
            Err(e) => {
                eprintln!("Error: {}", e.to_string());
                return;
            }
        }
    }
}

/// Rayon thread pool
pub struct RayonThreadPool {
    threads: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let threads = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build()?;
        Ok(RayonThreadPool { threads })
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.threads.spawn(move || {
            job();
        })
    }
}

// struct ThreadPool {
//     workers: Vec<Worker>,
//     sender: mpsc::Sender<Job>,
// }

// impl ThreadPool {
//     fn new(size: usize) -> Result<Self> {
//         let (sender, receiver) = mpsc::channel();
//         let receiver = Arc::new(Mutex::new(receiver));
//         let mut workers: Vec<Worker> = Vec::with_capacity(size);
//         for i in 0..size {
//             workers.push(Worker::new(i, receiver.clone()));
//         }
//         Ok(ThreadPool { workers, sender })
//     }

//     fn spawn<F>(&self, job: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         let job = Box::new(job);
//         self.sender.send(job).unwrap();
//     }
// }
