#[cfg(target_os = "linux")]
mod linux_impl {
    use std::collections::HashMap;
    use std::future::Future;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::unix::io::RawFd;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex, OnceLock, mpsc};
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use std::time::Duration;

    // --- Global Reactor State ---
    static EPOLL_FD: OnceLock<RawFd> = OnceLock::new();
    static WAKERS: OnceLock<Mutex<HashMap<RawFd, Waker>>> = OnceLock::new();

    fn get_epoll() -> RawFd {
        *EPOLL_FD.get_or_init(|| unsafe {
            let fd = libc::epoll_create1(libc::EPOLL_CLOEXEC);
            if fd < 0 {
                panic!("epoll_create1 failed");
            }
            fd
        })
    }

    fn get_wakers() -> &'static Mutex<HashMap<RawFd, Waker>> {
        WAKERS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    // --- Task Definition ---
    // A task wraps a Future. We use Mutex to allow polling via Arc.
    pub struct Task {
        pub future: Mutex<Pin<Box<dyn Future<Output = u64> + Send + Sync>>>,
    }

    // --- Executor (RunLoop) ---
    pub struct RunLoop {
        rx: mpsc::Receiver<Arc<Task>>,
        tx: mpsc::SyncSender<Arc<Task>>,
    }

    pub struct Spawner {
        tx: mpsc::SyncSender<Arc<Task>>,
    }

    impl Spawner {
        pub fn spawn<F>(&self, fut: F)
        where
            F: Future<Output = u64> + 'static + Send + Sync,
        {
            let task = Arc::new(Task {
                future: Mutex::new(Box::pin(fut)),
            });
            self.tx.send(task).expect("Channel closed");
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

        pub fn run(&self) {
            let mut events: [libc::epoll_event; 32] =
                [unsafe { MaybeUninit::zeroed().assume_init() }; 32];
            let epfd = get_epoll();

            loop {
                // 1. Process all available tasks in the channel
                loop {
                    match self.rx.try_recv() {
                        Ok(task) => {
                            // Create a waker that will reschedule this task
                            let waker = self.make_waker(task.clone());
                            let mut cx = Context::from_waker(&waker);

                            let mut future = task.future.lock().unwrap();
                            match future.as_mut().poll(&mut cx) {
                                Poll::Ready(_) => { /* Task done */ }
                                Poll::Pending => { /* Task will wake us later */ }
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => break, // Queue empty, go to reactor
                        Err(mpsc::TryRecvError::Disconnected) => return,
                    }
                }

                // 2. Wait for IO events (Reactor)
                // We wait indefinitely here because we have no other work.
                let n = unsafe { libc::epoll_wait(epfd, events.as_mut_ptr(), 32, -1) };

                if n < 0 {
                    let err = io::Error::last_os_error();
                    if err.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    panic!("epoll_wait failed");
                }

                // 3. Wake up tasks associated with ready FDs
                let mut wakers = get_wakers().lock().unwrap();
                for ev in events.into_iter() {
                    let fd = { ev.u64 as RawFd };
                    if let Some(waker) = wakers.remove(&fd) {
                        waker.wake();
                    }
                }
            }
        }

        fn make_waker(&self, task: Arc<Task>) -> Waker {
            let waker_data = Arc::new(WakerData {
                task,
                tx: self.tx.clone(),
            });
            waker_data.into_waker()
        }
    }

    // --- Waker Implementation ---
    struct WakerData {
        task: Arc<Task>,
        tx: mpsc::SyncSender<Arc<Task>>,
    }

    impl WakerData {
        fn into_waker(self: Arc<Self>) -> Waker {
            let raw = Arc::into_raw(self) as *const ();
            let vtable = &RawWakerVTable::new(
                Self::clone_waker,
                Self::wake,
                Self::wake_by_ref,
                Self::drop_waker,
            );
            unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
        }

        fn clone_waker(data: *const ()) -> RawWaker {
            let arc = unsafe { Arc::from_raw(data as *const WakerData) };
            // RawWaker is type-erased and doesn't manage ref-counts automatically.
            // We must manually bridge this gap:
            // 1. `arc.clone()` increments the reference count (e.g. 1 -> 2).
            // 2. `mem::forget` prevents the destructor from running on this clone.
            // This transfers the ownership of that extra reference count to the new RawWaker we are creating.
            //
            // If we didn't forget, the clone would drop immediately, the count would go back to 1,
            // and the new RawWaker would point to data without a corresponding reference count,
            // leading to a "use after free" error later.
            std::mem::forget(arc.clone());
            let ptr = Arc::into_raw(arc) as *const ();
            let vtable = &RawWakerVTable::new(
                Self::clone_waker,
                Self::wake,
                Self::wake_by_ref,
                Self::drop_waker,
            );
            RawWaker::new(ptr, vtable)
        }

        fn wake(data: *const ()) {
            let arc = unsafe { Arc::from_raw(data as *const WakerData) };
            let _ = arc.tx.send(arc.task.clone());
        }

        fn wake_by_ref(data: *const ()) {
            // If you don't do this, you will cause a double-free or use-after-free error.
            //
            // Arc::from_raw assumes it is taking full ownership of a reference. If you let this temporary
            // Arc drop normally at the end of the function, it will decrement the reference count. Since
            // wake_by_ref is only supposed to borrow the waker (not consume it), that decrement is
            // "stolen" from the caller, causing the memory to be freed while the caller still thinks it's
            // valid. ManuallyDrop prevents this accidental cleanup.
            let arc =
                std::mem::ManuallyDrop::new(unsafe { Arc::from_raw(data as *const WakerData) });
            let _ = arc.tx.send(arc.task.clone());
        }

        fn drop_waker(data: *const ()) {
            drop(unsafe { Arc::from_raw(data as *const WakerData) });
        }
    }

    // --- MyTask (Timer Future) ---
    pub struct MyTask {
        timerfd: RawFd,
        registered: bool,
    }

    impl MyTask {
        pub fn new(duration: Duration) -> Self {
            unsafe {
                let timerfd = libc::timerfd_create(
                    libc::CLOCK_MONOTONIC,
                    libc::TFD_NONBLOCK | libc::TFD_CLOEXEC,
                );
                if timerfd < 0 {
                    panic!("timerfd_create failed");
                }

                let it_value = libc::timespec {
                    tv_sec: duration.as_secs() as i64,
                    tv_nsec: duration.subsec_nanos() as i64,
                };
                let new_value = libc::itimerspec {
                    it_interval: libc::timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    },
                    it_value,
                };

                if libc::timerfd_settime(timerfd, 0, &new_value, std::ptr::null_mut()) < 0 {
                    panic!("timerfd_settime failed");
                }

                MyTask {
                    timerfd,
                    registered: false,
                }
            }
        }
    }

    impl Drop for MyTask {
        fn drop(&mut self) {
            unsafe { libc::close(self.timerfd) };
        }
    }

    impl Future for MyTask {
        type Output = u64;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let fd = self.timerfd;
            let mut buf = 0u64;

            // 1. Try to read from timerfd
            let res = unsafe { libc::read(fd, &mut buf as *mut _ as *mut _, 8) };

            if res == 8 {
                Poll::Ready(buf)
            } else {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock {
                    // 2. Not ready, register with epoll and save waker
                    if !self.registered {
                        let epfd = get_epoll();
                        let mut ev = libc::epoll_event {
                            events: libc::EPOLLIN as u32 | libc::EPOLLONESHOT as u32,
                            u64: fd as u64,
                        };
                        unsafe {
                            libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, fd, &mut ev);
                        }
                        self.registered = true;
                    } else {
                        // Re-arm one-shot
                        let epfd = get_epoll();
                        let mut ev = libc::epoll_event {
                            events: libc::EPOLLIN as u32 | libc::EPOLLONESHOT as u32,
                            u64: fd as u64,
                        };
                        unsafe {
                            libc::epoll_ctl(epfd, libc::EPOLL_CTL_MOD, fd, &mut ev);
                        }
                    }

                    // Store waker so Reactor can wake us
                    let mut wakers = get_wakers().lock().unwrap();
                    wakers.insert(fd, cx.waker().clone());

                    Poll::Pending
                } else {
                    panic!("read timerfd failed");
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn main() {
    use linux_impl::{MyTask, RunLoop};

    let runloop = RunLoop::new(10);
    let spawner = runloop.spawner();

    spawner.spawn(async {
        use std::time::Duration;

        println!("Task: sleep 1s");
        let _ = MyTask::new(Duration::from_secs(1)).await;
        println!("Task: woke up, sleep 2s");
        let _ = MyTask::new(Duration::from_secs(2)).await;
        println!("Task: woke up again, done.");
        0
    });

    runloop.run();
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("This example requires Linux (epoll/timerfd).");
    println!("Please run on a Linux machine to see the async runtime in action.");
}
