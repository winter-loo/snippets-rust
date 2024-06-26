use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

mod chatgpt;
use chatgpt::ChatGPT;
use futures::channel::mpsc;
use futures::SinkExt;

pub struct Chati {
    gpt: ChatGPT,
    he_said_tx: mpsc::UnboundedSender<String>,
    he_said_rx: mpsc::UnboundedReceiver<String>,
}

impl Chati {
    pub async fn new() -> Self {
        let gpt = ChatGPT::new().await;
        let (he_said_tx, he_said_rx) = mpsc::unbounded();
        Chati {
            gpt,
            he_said_tx,
            he_said_rx,
        }
    }

    pub async fn new_converstation(&mut self) {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_tx = Arc::clone(&flag);
        let flag_rx = Arc::clone(&flag);
        
        tokio::task::spawn(async move {
            loop {
                if flag_rx.load(Ordering::Acquire) {
                    println!("Flag is set, task can proceed");
                    break;
                } else {
                    println!("Flag is not set, checking again...");
                    // To prevent busy-waiting, you can sleep for a short duration
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }

            if let Err(error) = chati::util::listen_webpage_stream_data(
                9222,
                "https://chatgpt.com/",
                0,
                "https://chatgpt.com/backend-anon/conversation",
                |data| {
                    chati::openai::assistant_sse(data, |stream_msg, ended| {
                        self.he_said_tx.send(stream_msg.to_string());
                    })
                },
            )
            .await
            {
                println!("listen_webpage_stream_data: {error:#?}");
            }
        });

        self.gpt.new_session(flag_tx).await;
    }

    pub async fn isaid(&self, said: &str) -> String {
        if let Err(error) = self.gpt.send_my_said(said).await {
            println!("chatgpt.startup: {error:#?}");
        }
        while let Some(words) = self.he_said_rx.next().await {
            return words;
        }
    }
}

#[tokio::main]
async fn main() {
    let ci = Chati:new().await;

    let what_i_said = "hello world";
    println!("I SAID: {what_i_said}");

    let hesaid = ci.isaid(what_i_said).await.into_stream();
    print!("HE SAID: ");
    while let Some(words) = hesaid.next().await {
        print!("{words}");
    }
    println!();
}