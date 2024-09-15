use tokio;
use std::future::Future;

struct Empty {}

impl Future for Empty {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        println!("poll in progress...");
        std::task::Poll::Pending
    }

}

#[tokio::main(flavor="current_thread")]
async fn main() {
    let fut = Empty {};
    println!("awaiting fut...");
    fut.await;
    println!("awaiting fut...DONE");
}
