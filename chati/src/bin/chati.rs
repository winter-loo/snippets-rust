use chati::chati::Chati;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use env_logger;
use log::{debug, info};
use std::io::Write;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut ci = Chati::new().await;

    ci.new_converstation(true).await;

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();

        let mut i_said = String::new();
        std::io::stdin().read_line(&mut i_said).unwrap();

        if i_said.trim() == "\\q" {
            break;
        }
        if i_said.trim() == "" {
            continue;
        }
        ensure_responded(&mut ci, &i_said).await;
    }

    println!("\nbye!");

    ci.end().await;
}

async fn ensure_responded(ci: &mut Chati, isaid: &str) {
    loop {
        // chati::util::pause_force().await;
        debug!("I SAID: {isaid}");

        ci.isaid(isaid).await;

        debug!("HE SAID: ");

        let repeat = Arc::new(AtomicBool::new(false));
        ci.hesaid(|words| {
            let repeat = Arc::clone(&repeat);
            async move {
                match words {
                    Some(words) => {
                        print!("{words}");
                        std::io::stdout().flush().unwrap();
                    }
                    None => {
                        info!("he said nothing. I will repeat my said");
                        repeat.store(true, Ordering::Relaxed);
                    }
                }
            }
        })
        .await;

        if !repeat.load(Ordering::Relaxed) {
            println!();
            std::io::stdout().flush().unwrap();
            break;
        }
    }
}
