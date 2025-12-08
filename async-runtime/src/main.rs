use std::sync::mpsc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

type Task = std::pin::Pin<Box<dyn Future<Output = u64>>>;
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
        F: Future<Output = u64> + 'static,
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
                    let waker = unsafe { Waker::new(std::ptr::null(), &VTABLE) };
                    let mut ctx = Context::from_waker(&waker);

                    match task.as_mut().poll(&mut ctx) {
                        Poll::Ready(_) => {
                            println!("task ready");
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
    sleep_time: std::time::Duration,
    start_time: std::time::Instant,
}

impl MyTask {
    fn new(sleep_time: std::time::Duration) -> Self {
        Self {
            sleep_time,
            start_time: std::time::Instant::now(),
        }
    }
}

impl Future for MyTask {
    type Output = u64;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if std::time::Instant::now() - self.start_time < self.sleep_time {
            Poll::Pending
        } else {
            std::task::Poll::Ready((std::time::Instant::now() - self.start_time).as_secs())
        }
    }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(std::ptr::null(), &VTABLE),
    // `wake` does nothing
    |_| {},
    // `wake_by_ref` does nothing
    |_| {},
    // Dropping does nothing as we don't allocate anything
    |_| {},
);

fn main() {
    let t = MyTask::new(Duration::from_secs(10));

    let mut runloop = RunLoop::new(20);
    let spawner = runloop.spawner();
    spawner.spawn(t);

    runloop.run();
}
