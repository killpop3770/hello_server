use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
pub enum ThreadPoolError {
    ThreadCreationError(String),
    WorkerCreationError(String),
}

impl Display for ThreadPoolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    handler: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Result<Worker, ThreadPoolError> {
        let builder = thread::Builder::new();
        let handler_result = builder.spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        match handler_result {
            Ok(handler) => Ok(Worker { id, handler: Some(handler) }),
            Err(err) => Err(ThreadPoolError::WorkerCreationError(format!("Cannot create new thread: {err}")))
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Result<ThreadPool, ThreadPoolError> {
        if size > 0 {
            let (sender, receiver) = channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);
            for id in 0..size {
                // create some threads and store them in the vector
                if let Ok(w) = Worker::new(id, Arc::clone(&receiver)) {
                    workers.push(w);
                }
            }
            Ok(ThreadPool { workers, sender: Some(sender) })
        } else {
            Err(ThreadPoolError::ThreadCreationError(String::from("Low number of threads!")))
        }
    }

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

            if let Some(hanldler) = worker.handler.take() {
                hanldler.join().unwrap();
            }
        }
    }
}