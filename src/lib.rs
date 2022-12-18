use std::collections::VecDeque;
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    size: usize,
    threads: Vec<Worker>,
    queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>>,
}

struct Worker { 
    id: usize,
    thread: JoinHandle<()>,
}

impl ThreadPool {
    /// Creates a new ThreadPool
    /// 
    /// Creates a thread pool with specified number of threads
    /// 
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let mut threads = Vec::with_capacity(size);
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        for worker_id in 0..size {
            threads.push(Worker::new(worker_id, Arc::clone(&queue)));
        }

        ThreadPool { size, threads, queue }
    }

    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.lock().unwrap().push_back(Box::new(f));
    }
}

impl Worker {
    fn new(id: usize, queue: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>>) -> Worker {
        let thread = thread::spawn(move || {
            while let Ok(mut queue) = queue.lock() {
                if let Some(closure) = queue.pop_front() {
                    drop(queue);
                    closure();
                }
            }
        });
        
        Worker { 
            id, 
            thread,
        }
    }
} 
