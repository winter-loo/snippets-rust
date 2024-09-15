use tokio::{self, time::Sleep};
use std::pin::Pin;
use std::future::Future;
use std::borrow::BorrowMut;

struct MyFuture {
    i: usize,
    sleep_fut: Box<Sleep>,
}

impl MyFuture {
    fn new(i: usize) -> Self {
        MyFuture {
            i,
            sleep_fut: Box::new(tokio::time::sleep(tokio::time::Duration::from_secs(1))),
        }
    }
}

impl Future for MyFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("poll in progress...{}", self.i);
        let this = self.get_mut();
        this.i += 1;

        let sleep_fut: Pin<&mut Sleep> = unsafe { std::pin::Pin::new_unchecked(this.sleep_fut.borrow_mut()) };

        match sleep_fut.poll(cx) {
            std::task::Poll::Ready(_) => {
                if this.i >= 5 {
                    std::task::Poll::Ready(())
                } else {
                    this.sleep_fut = Box::new(tokio::time::sleep(tokio::time::Duration::from_secs(1)));
                    let sleep_fut: Pin<&mut Sleep> = unsafe { std::pin::Pin::new_unchecked(this.sleep_fut.borrow_mut()) };
                    sleep_fut.poll(cx)
                }
            },
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[tokio::main(flavor="current_thread")]
async fn main() {
    let fut = MyFuture::new(0);
    println!("awaiting fut...");
    fut.await;
    println!("awaiting fut...DONE");
}
