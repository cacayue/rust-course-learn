use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use thread::JoinHandle;

pub const GET: &[u8] = b"GET / HTTP/1.1\r\n";
pub const SLEEP: &[u8] = b"GET /sleep HTTP/1.1\r\n";

enum Message {
    NewJob(Job),
    Terminate,
}

pub enum HtmlFile {
    OK,
    SLEEP,
    NotFound,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// 创建线程池
    ///
    /// 线程池中线程的数量
    ///
    ///  # Panics
    ///
    ///   `new`函数在count为0时会panic
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            let receiver_clone = Arc::clone(&receiver);
            let work = Worker::new(id, receiver_clone);
            workers.push(work);
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
		println!("Sending terminate message to all workers.");

		for _ in &self.workers {
			self.sender.send(Message::Terminate).unwrap();
		}

		print!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job();
                }
                Message::Terminate => {
                    println!("Worker {} told to terminate; executing.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
