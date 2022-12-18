use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    size: usize,
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool
    /// 
    /// Creates a thread pool with specified number of threads
    /// 
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for worker_id in 0..size {
            workers.push(Worker::new(worker_id, Arc::clone(&receiver)));
        }

        ThreadPool { size, workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker { 
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            if let Ok(job) = job {
                println!("Worker {id} received job.");

                job();
            } else {
                println!("No longer receiving messages! Worker {id} will shut down.");
                break;
            }
        });
        
        Worker { 
            id, 
            thread: Some(thread),
        }
    }
} 
