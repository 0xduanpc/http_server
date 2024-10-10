use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// 线程池结构体，管理一组线程
pub struct ThreadPool {
    workers: Vec<Worker>,  // 保存所有工作线程
    sender: mpsc::Sender<Job>,  // 任务的发送端
}

type Job = Box<dyn FnOnce() + Send + 'static>;  // 任务是一种可以在线程中执行的闭包

impl ThreadPool {
    /// 创建一个新的线程池
    ///
    /// 线程池中线程的数量将根据 `size` 参数决定。
    ///
    /// # Panics
    ///
    /// 如果 `size` 为 0，则会 panic。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);  // size 必须大于 0

        let (sender, receiver) = mpsc::channel();  // 创建任务通道
        let receiver = Arc::new(Mutex::new(receiver));  // 用 Arc 和 Mutex 包装，使其可以在线程间共享

        let mut workers = Vec::with_capacity(size);  // 提前为线程池分配足够的空间

        // 创建指定数量的工作线程
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// 将任务提交给线程池
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);  // 将任务封装到 Box 中，满足 trait object
        self.sender.send(job).unwrap();  // 通过任务通道发送任务
    }
}

/// 工作线程结构体
struct Worker {
    id: usize,  // 每个线程都有一个唯一的 id
    thread: Option<thread::JoinHandle<()>>,  // 线程句柄
}

impl Worker {
    /// 创建一个新的工作线程，并从任务队列中执行任务
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();  // 从通道中接收任务

            println!("Worker {} got a job; executing.", id);

            job();  // 执行任务
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}