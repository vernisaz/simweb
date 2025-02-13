use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
// add a thread execution time out and kill the execution after

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool { workers, sender: Some(sender) }
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
            eprintln!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl  Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // using while or if let will block other threads from an execution
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    eprintln!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    eprintln!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}
