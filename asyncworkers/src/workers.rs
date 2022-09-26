
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Asynchronous Worker Pool
///
/// A worker pool for executing jobs asynchronously.
///
/// ```
/// use asyncworkers::*;
///
/// let mut w = Workers::new(3);
/// let arr = vec![1,2,3,4,5,6,7,8,9,10];
/// let var = arr.clone();
/// w.execute(move || {
///     println!("Executing work 1!");
///     for i in &var[..4] {
///         println!("Printing {}", i);
///     }
/// });
/// ```
///
pub struct Workers {
    pool: Vec<Option<thread::JoinHandle<()>>>,
    sender: Option<Sender<Work>>
}

/// Generic work definition
type Work = Box<dyn FnOnce() + Send + 'static>;

impl Workers {
    /// Create a new worker pool of given size
    pub fn new(sz: usize) -> Self {
        // create a thread pool
        let mut pool = Vec::with_capacity(sz);
        // create job channel
        let (tx, rx): (Sender<Work>, Receiver<Work>) = mpsc::channel();
        // since reciever will be used from multiple threads
        // from the pool, wrap it in Arc+Mutex for synchronized
        // access
        let rx = Arc::new(Mutex::new(rx));

        // create the threads in the pool
        for idx in 0..sz {
            let receiver = Arc::clone(&rx);
            let worker = thread::spawn( move || {
                println!("Worker {}: Ready", idx);
                loop {
                    // receive work and execute; exit if channel is closed
                    match receiver.lock().unwrap().recv() {
                        Ok(work) => {
                            #[cfg(Debug)]
                            println!("Worker {}: Executing...", idx);
                            work();
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    }
                }

            });
            // add thread to pool
            pool.push(Some(worker));
        }
        Workers { pool, sender: Some(tx) }
    }

    pub fn execute<F>(&mut self, work: F)
        where F: FnOnce() + Send + 'static
    {
        // send job in the channel; first one to receive will execute
        self.sender.as_ref().unwrap().send(Box::new(work)).unwrap();
    }
}

/// Graceful shutdown and cleanup
impl Drop for Workers {
    fn drop(&mut self) {
        // Close the channel
        drop(self.sender.take());

        // wait for all threads to exit
        for w in &mut self.pool {
            w.take().unwrap().join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workers() {
        let mut w = Workers::new(3);
        let arr = vec![1,2,3,4,5,6,7,8,9,10];
        let var = arr.clone();
        w.execute(move || {
            println!("Executing work 1!");
            for i in &var[..4] {
                println!("Printing {}", i);
            }
        });
        let var = arr.clone();
        w.execute(move || {
            println!("Executing work 2!");
            for i in &var[5..] {
                println!("Printing {}", i);
            }
        });
        let var = arr.clone();
        w.execute(move || {
            println!("Executing work 3!");
            for i in &var[2..4] {
                println!("Printing {}", i);
            }
        });
        let var = arr.clone();
        w.execute(move || {
            println!("Executing work 4!");
            for i in &var[3..7] {
                println!("Printing {}", i);
            }
        });
        let var = arr.clone();
        w.execute(move || {
            println!("Executing work 5");
            for i in &var[5..9] {
                println!("Printing {}", i);
            }
        });
    }
}