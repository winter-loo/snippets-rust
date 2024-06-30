use chati::chati::Chati;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let mut ci = Chati::new().await;

    ci.new_converstation(true).await;

    let mut i_said = "hello";
    ensure_responded(&mut ci, &i_said).await;

    i_said = "are you ok?";
    ensure_responded(&mut ci, &i_said).await;

    println!("\ndone");
    tokio::io::stdout().flush().await.unwrap();

    ci.end().await;
}

async fn ensure_responded(ci: &mut Chati, isaid: &str) {
    loop {
        chati::util::pause_force().await;
        println!("I SAID: {isaid}");
        tokio::io::stdout().flush().await.unwrap();

        ci.isaid(isaid).await;

        print!("HE SAID: ");
        tokio::io::stdout().flush().await.unwrap();

        let repeat = Arc::new(AtomicBool::new(false));
        ci.hesaid(|words| {
            let repeat = Arc::clone(&repeat);
            async move {
                match words {
                    Some(words) => {
                        print!("{words}");
                        let _ = tokio::io::stdout().flush().await;
                    }
                    None => {
                        println!("he said nothing. I will repeat my said");
                        repeat.store(true, Ordering::Relaxed);
                    }
                }
            }
        })
        .await;

        if !repeat.load(Ordering::Relaxed) {
            println!();
            tokio::io::stdout().flush().await.unwrap();
            break;
        }
    }
}
