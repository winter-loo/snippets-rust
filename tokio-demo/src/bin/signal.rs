use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    let mut usr1 = signal(SignalKind::user_defined1()).unwrap();
    let mut usr2 = signal(SignalKind::user_defined2()).unwrap();

    loop {
        tokio::select! {
            _ = usr1.recv() => println!("signal USR1 is received"),
            _ = usr2.recv() => println!("signal USR2 is received"),
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::signal::unix::{signal, SignalKind};

    #[tokio::test]
    async fn one_signal_stream() {
        let mut usr1 = signal(SignalKind::user_defined1()).unwrap();
        loop {
            usr1.recv().await;
            println!("signal USR1 is received");
        }
    }
}
