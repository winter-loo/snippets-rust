use std::pin::Pin;
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
                    // task.as_mut().poll();
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
        let waker = cx.waker();
        let runloop = unsafe { waker.data() as *const RunLoop };
        if std::time::Instant::now() - self.start_time < self.sleep_time {
            Poll::Pending
        } else {
            std::task::Poll::Ready((std::time::Instant::now() - self.start_time).as_secs())
            // waker.wake();
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
    let mut t = MyTask::new(Duration::from_secs(10));
    let t = Pin::new(&mut t);

    let mut runloop = RunLoop::new(20);
    let spawner = runloop.spawner();
    spawner.spawn(async { 1 });

    runloop.run();

    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);

    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut c = Context::from_waker(&waker);

    // loop {
    match t.poll(&mut c) {
        Poll::Ready(v) => println!("DONE: {v:?}"),
        Poll::Pending => println!("pending"),
    }
    // }
}
