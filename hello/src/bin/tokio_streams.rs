use tokio_stream::StreamExt;

struct MyStream {
    counter: u32,
}

impl tokio_stream::Stream for MyStream {
    type Item = u32;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Option<Self::Item>> {
        self.counter += 1;
        // Pretend there's some work here
        for _ in 0..1000 {
        }
        if self.counter < 1000 {
            std::task::Poll::Ready(Some(self.counter))
        } else {
            std::task::Poll::Ready(None)
        }
    }
}

async fn ticker() {
    loop {
        print!("T");
        tokio::task::yield_now().await;
    }
}

async fn streamer() -> MyStream {
    let stream = MyStream { counter: 0 };
    stream
}

#[tokio::main]
async fn main() {
    tokio::spawn(ticker());
    let mut mystream = streamer();
    while mystream.next().await.is_some() {
        print!(".");
    }
}