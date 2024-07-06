use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::chatgpt::ChatGPT;
use crate::openai;
use crate::util;

use log::{debug, error};

pub struct Chati {
    gpt: ChatGPT,
    // he could say nothing
    he_said_tx: mpsc::UnboundedSender<Option<String>>,
    he_said_rx: mpsc::UnboundedReceiver<Option<String>>,
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

    pub async fn new_converstation(&mut self, auto_login: bool) {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_tx = Arc::clone(&flag);
        let flag_rx = Arc::clone(&flag);

        let he_said_tx = self.he_said_tx.clone();
        tokio::task::spawn(async move {
            loop {
                if flag_rx.load(Ordering::Acquire) {
                    debug!("Flag is set, task can proceed");
                    break;
                } else {
                    debug!("Flag is not set, checking again...");
                    // To prevent busy-waiting, you can sleep for a short duration
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }

            let he_said_tx = he_said_tx.clone();
            // It could send two or more None in a run when respond to one chat message.
            // The Situation is: when you send your first message, chatgpt.com could
            // respond with an http 403 error. Then, you click the button "重新生成",
            // chatgpt.com responds again with an http 403 error. This redundant message
            // will result in sending the same user message again. Hence, `has_said_none`
            // variable ensures at most one deliverary semantics.
            let mut has_said_none = false;
            if let Err(error) = util::listen_webpage_stream_data(
                9222,
                "https://chatgpt.com/",
                0,
                "https://chatgpt.com/backend-anon/conversation",
                |data| match data {
                    Some(data) => {
                        has_said_none = false;
                        openai::assistant_sse(data, |stream_msg, ended| {
                            let words = stream_msg.to_string();
                            if let Err(error) = he_said_tx.send(Some(words)) {
                                error!("send response data to inner channel: {error:#?}");
                            }
                            if ended {
                                if let Err(error) =
                                    he_said_tx.send(Some(Self::WORDS_ENDED.to_string()))
                                {
                                    error!("send response data to inner channel: {error:#?}");
                                }
                            }
                        });
                    }
                    None => {
                        if !has_said_none {
                            has_said_none = true;
                            if let Err(error) = he_said_tx.send(None) {
                                error!("send response data to inner channel: {error:#?}");
                            }
                        }
                    }
                },
            )
            .await
            {
                error!("error on listen_webpage_stream_data: {error:#?}");
            }
        });

        if auto_login {
            self.gpt.new_session(flag_tx).await;
        } else {
            self.gpt.wait_for_chatbox(flag_tx).await;
        }
    }

    pub async fn isaid(&mut self, said: &str) {
        self.gpt.send_my_said(said).await;
    }

    pub async fn hesaid<F, Fut>(&mut self, mut out: F)
    where
        F: FnMut(Option<String>) -> Fut,
        Fut: futures::Future<Output = ()>,
    {
        while let Some(words) = self.he_said_rx.recv().await {
            match words {
                Some(words) => {
                    if words == Self::WORDS_ENDED {
                        out(Some("\n".to_string()));
                        break;
                    }
                    out(Some(words)).await;
                }
                None => {
                    out(None).await;
                    break;
                }
            }
        }
    }

    pub async fn end(self) {
        if let Err(error) = self.gpt.close().await {
            error!("chatgpt.close: {error:#?}");
        }
    }
}
