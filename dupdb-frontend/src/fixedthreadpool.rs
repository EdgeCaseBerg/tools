use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};


type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
	id: usize,
	thread: JoinHandle<()>,
}

impl Worker {
	/// Can panic if there are no threads available to spawn a new thread
	fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
		Worker {
			id,
			thread: thread::spawn(move || {
				loop {
					let message = receiver
						.lock().expect("Failed to acquire lock, did another thread poison the state?")
						.recv();

					match message {
						Ok(job) => {
							println!("Worker {id} got a job; executing.");
							job();
						},
						Err(_) => {
							println!("Worker {id} disconnected; shutting down");
							break;
						}
					}
				}
			})
		}
	}
}

pub struct FixedThreadPool {
	workers: Vec<Worker>,
	sender: Option<Sender<Job>>,
}

impl FixedThreadPool {
	/// Create a new fixed size thread pool
	///
	/// Size is the number of threads in the pool
	///
	/// # Panics
	///
	/// The new function will panic if the size is 0
	pub fn new(size: usize) -> Self {
		assert!(size > 0);

		let (sender, receiver) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));
		
		let mut workers = Vec::with_capacity(size);
		for id in 0..size {
			workers.push(Worker::new(id, Arc::clone(&receiver)));
		}

		FixedThreadPool { workers, sender: Some(sender) }
	}

	pub fn execute<F>(&mut self, thunk: F)
	where F: FnOnce() + Send + 'static,
	{
		let job = Box::new(thunk);
		let send_result = self.sender.as_ref()
			.expect("Attempted to execute job after sender has been dropped")
			.send(job);
		if let Err(error) = send_result {
			eprintln!("Could not send job {error}");
			let mut roman_imperial_method_of_succession = FixedThreadPool::new(self.workers.capacity());
			self.workers = std::mem::take(&mut roman_imperial_method_of_succession.workers);
			self.sender = roman_imperial_method_of_succession.sender.take();
		}
	}
}

impl Drop for FixedThreadPool {
	fn drop(&mut self) {
		// Sender must be explicitly dropped in order to 
		// ensure that the worker threads actually stop looping.
		drop(self.sender.take());
		for worker in self.workers.drain(..) {
			println!("Shutting down worker {:?}", worker.id);
			match worker.thread.join() {
				Ok(_) => println!("Successfully shut down worker {:?}", worker.id),
				Err(error) => eprint!("Could not shut down worker properly {:?} {:?}", worker.id, error),
			}
		}
	}
}