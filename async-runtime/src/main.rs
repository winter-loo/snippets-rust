use std::sync::{mpsc, Arc};
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
                    // Create a Spawner from the current runloop's tx.
                    // This Spawner will be given to the Waker.
                    let spawner_instance = Spawner {
                        tx: self.tx.clone(),
                    };

                    let waker = unsafe {
                        Waker::new(&spawner_instance as *const Spawner as *const (), &VTABLE)
                    };
                    let mut ctx = Context::from_waker(&waker);

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
    inner: Arc<Inner>,
}

struct Inner {
    sleep_time: std::time::Duration,
    start_time: std::time::Instant,
}

impl MyTask {
    fn new(sleep_time: std::time::Duration) -> Self {
        Self {
            inner: Arc::new(Inner {
                sleep_time,
                start_time: std::time::Instant::now(),
            }),
        }
    }
}

impl Future for MyTask {
    type Output = u64;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if std::time::Instant::now() - self.inner.start_time < self.inner.sleep_time {
            //
            // We need somehow push the task into runloop
            //
            // And here, we pass into the task spawner as the any data field and use it
            // to send back the task
            //
            // lightweight task copy
            let new_task = MyTask {
                inner: Arc::clone(&self.inner),
            };
            let spawner = unsafe { &*(ctx.waker().data() as *const Spawner) };
            spawner.spawn(new_task);
            Poll::Pending
        } else {
            std::task::Poll::Ready((std::time::Instant::now() - self.inner.start_time).as_secs())
        }
    }
}

// Helper function to get `Arc<Spawner>` from `*const ()`
unsafe fn data_to_arc_spawner(data: *const ()) -> Spawner {
    let s = unsafe { &*(data as *const Spawner) };
    Spawner { tx: s.tx.clone() }
}

fn clone_arc_spawner(data: *const ()) -> RawWaker {
    let spawner = unsafe { data_to_arc_spawner(data) };
    RawWaker::new(&spawner as *const Spawner as *const (), &VTABLE)
}

fn wake_arc_spawner(data: *const ()) {
    println!("Wake called!");
    unsafe {
        let _ = data_to_arc_spawner(data); // Drop the Arc, decrement ref count
    }
}

fn wake_by_ref_arc_spawner(data: *const ()) {
    println!("Wake by ref called!");
}

fn drop_arc_spawner(data: *const ()) {
    println!("Drop called!");
    unsafe {
        let _ = data_to_arc_spawner(data); // Drop the Arc, decrement ref count
    }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_arc_spawner,
    wake_arc_spawner,
    wake_by_ref_arc_spawner,
    drop_arc_spawner,
);

fn main() {
    let mut runloop = RunLoop::new(20);
    let spawner = runloop.spawner();
    let t = MyTask::new(Duration::from_secs(1));

    spawner.spawn(t);

    runloop.run();
}
