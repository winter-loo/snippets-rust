use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::chatgpt::ChatGPT;
use crate::openai;
use crate::util;

pub struct Chati {
    gpt: ChatGPT,
    he_said_tx: mpsc::UnboundedSender<String>,
    he_said_rx: mpsc::UnboundedReceiver<String>,
}

impl Chati {
    const WORDS_ENDED: &'static str = "\n\n\n\n\n\n";

    pub async fn new() -> Self {
        let gpt = ChatGPT::new().await;
        let (he_said_tx, he_said_rx) = mpsc::unbounded_channel();
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

        let he_said_tx = self.he_said_tx.clone();
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

            let he_said_tx = he_said_tx.clone();
            if let Err(error) = util::listen_webpage_stream_data(
                9222,
                "https://chatgpt.com/",
                0,
                "https://chatgpt.com/backend-anon/conversation",
                |data| {
                    openai::assistant_sse(data, |stream_msg, ended| {
                        let words = stream_msg.to_string();
                        if let Err(error) = he_said_tx.send(words) {
                            println!("send response data to inner channel: {error:#?}");
                        }
                        if ended {
                            if let Err(error) = he_said_tx.send(Self::WORDS_ENDED.to_string()) {
                                println!("send response data to inner channel: {error:#?}");
                            }
                        }
                    })
                },
            )
            .await
            {
                println!("error on listen_webpage_stream_data: {error:#?}");
            }
        });

        self.gpt.new_session(flag_tx).await;
    }

    pub async fn isaid(&self, said: &str) {
        if let Err(error) = self.gpt.send_my_said(said).await {
            println!("chatgpt.startup: {error:#?}");
        }
    }

    pub async fn hesaid<F, Fut>(&mut self, mut out: F) 
    where 
        F: FnMut(String) -> Fut,
        Fut: futures::Future<Output = ()>, 
    {
        while let Some(words) = self.he_said_rx.recv().await {
            if words == Self::WORDS_ENDED {
                break;
            }
            out(words).await;
        }
    }

    pub async fn end(self) {
        if let Err(error) = self.gpt.close().await {
            println!("chatgpt.close: {error:#?}");
        }
    }
}
