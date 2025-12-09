use std::sync::{Arc, Mutex, mpsc};
use std::task::Poll;
use std::time::Duration;

// Send + Sync are added as the task will be sent back and forth through thread
type Task = std::pin::Pin<Box<dyn Future<Output = u64> + Send + Sync>>;
struct RunLoop {
    rx: mpsc::Receiver<Task>,
    tx: mpsc::SyncSender<Task>,
}

struct Spawner {
    tx: mpsc::SyncSender<Task>,
}

impl Spawner {
    fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = u64> + 'static + Send + Sync,
    {
        let x = Box::into_pin(Box::new(fut));
        let _ = self.tx.try_send(x);
    }
}

impl RunLoop {
    pub fn new(size: usize) -> Self {
        let (tx, rx) = mpsc::sync_channel(size);
        RunLoop { rx, tx }
    }
    pub fn spawner(&self) -> Spawner {
        Spawner {
            tx: self.tx.clone(),
        }
    }
    pub fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(mut task) => {
                    println!("received a task");
                    let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
                    match task.as_mut().poll(&mut ctx) {
                        Poll::Ready(_) => {
                            println!("task ready");
                            return;
                        }
                        Poll::Pending => {
                            println!("task pending");
                        }
                    }
                }
                Err(_) => {
                    panic!("task receiving error");
                }
            }
        }
    }
}

struct MyTask {
    // use reference counting to avoid copy data
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    ready: bool,
}

impl MyTask {
    fn new(sleep_time: std::time::Duration, spawner: &Spawner) -> Self {
        let t = MyTask {
            inner: Arc::new(Mutex::new(Inner { ready: false })),
        };

        let inner = Arc::clone(&t.inner);
        let tx = spawner.tx.clone();
        // start a thread to do the hard work
        // and signal the runloop when work done
        let _jh = std::thread::spawn(move || {
            std::thread::sleep(sleep_time);
            {
                let mut inner = inner.lock().unwrap();
                inner.ready = true;
            }
            // future(task) sent in this thread and received in the main thread
            let _ = tx.try_send(Box::pin(Box::new(MyTask {
                inner: Arc::clone(&inner),
            })));
        });
        t
    }
}

impl Future for MyTask {
    type Output = u64;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let inner = self.inner.lock().unwrap();
        if !inner.ready {
            // move task signaling to the thread created in the constructor
            Poll::Pending
        } else {
            std::task::Poll::Ready(std::time::Instant::now().elapsed().as_secs())
        }
    }
}

fn main() {
    let mut runloop = RunLoop::new(20);
    let spawner = runloop.spawner();
    let t = MyTask::new(Duration::from_secs(1), &spawner);

    spawner.spawn(t);

    runloop.run();
}
