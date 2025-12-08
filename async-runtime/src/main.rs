use std::ptr::null;
use std::task::{Context, RawWaker, Waker, RawWakerVTable, Poll};
use std::pin::Pin;
use std::time::Duration;

struct RunLoop {

}

impl RunLoop {
    
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

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let waker = cx.waker();
        if std::time::Instant::now() - self.start_time < self.sleep_time {
            Poll::Pending
        } else {
            std::task::Poll::Ready((std::time::Instant::now() - self.start_time).as_secs())
        }
    }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| { RawWaker::new(null(), &VTABLE) },
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

    let raw_waker = RawWaker::new(null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut c = Context::from_waker(&waker);

    match t.poll(&mut c) {
        Poll::Ready(v) => println!("DONE: {v:?}"),
        Poll::Pending => println!("pending"),
    }
}
