use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// A pool of threads where each thread exectues tasks independently
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    /// Execute the given closure in one of the threads from ThreadPool.
    ///
    /// # Panics
    ///
    /// The `execute` function will panic if no threads are present in the ThreadPool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .send(Message::NewJob(job))
            .expect("Failed to send the task to thread for execution");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        for _ in &self.workers {
            self.sender
                .send(Message::Terminate)
                .expect("Failed to send Terminate signal to the threads");
        }

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.handle.take() {
                thread
                    .join()
                    .expect("Failed to wait for the thread to finish");
            }
        }
    }
}

// Common term in pooling implementations
struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("Failed to acquire mutext lock")
                .recv()
                .expect("Sending channel hung up");
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Woker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            handle: Some(handle),
        }
    }
}
