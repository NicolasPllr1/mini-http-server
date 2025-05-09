use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    #[must_use]
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            if let Ok(job) = message {
                println!("Worker {id} got a job; executing.");
                job();
            } else {
                println!("Worker {id} disconnected; shutting down.");
                break;
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Here is the new process that will happen when we create a `ThreadPool`. We’ll implement the code that sends the closure to the thread after we have Worker set up in this way:
//
// Define a Worker::new function that takes an id number and returns a Worker instance that holds the id and a thread spawned with an empty closure.
// In ThreadPool::new, use the for loop counter to generate an id, create a new Worker with that id, and store the worker in the vector.

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    #[must_use]
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (tx, rx) = mpsc::channel(); // transmitter (tx), receiver (rx)
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }

        ThreadPool {
            workers,
            sender: Some(tx),
        }
    }
    /// Execute a task on the threadpool.
    /// Creates a `Job` from a task `f` and dispatch it to a worker which will carry-on the job
    /// execution.
    ///
    /// # Panics
    ///
    /// Sending the job to a worker can panic.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
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
