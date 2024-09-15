use tokio;
use std::future::Future;

struct MyFuture {
    i: usize,
}

impl Future for MyFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("poll in progress...{}", self.i);
        let this = self.get_mut();
        this.i += 1;

        let waker_clone = cx.waker().clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            waker_clone.wake_by_ref();
        });

        if this.i >= 5 {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

#[tokio::main(flavor="current_thread")]
async fn main() {
    let fut = MyFuture { i: 0 };
    println!("awaiting fut...");
    fut.await;
    println!("awaiting fut...DONE");
}
