use tokio;
use std::future::Future;

struct Empty {}

impl Future for Empty {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("poll in progress...");
        std::task::Poll::Ready(())
    }

}

#[tokio::main(flavor="current_thread")]
async fn main() {
    let fut = Empty {};
    println!("awaiting fut...");
    // An await in an async function typically results in a poll call on the future.
    fut.await;
    println!("awaiting fut...DONE");
}
