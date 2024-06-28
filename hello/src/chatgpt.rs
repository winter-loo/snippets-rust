#![allow(dead_code)]

use fantoccini::{client::*, elements::*, ClientBuilder, Locator};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio;

use crate::util;

enum WebState {
    ChatReady,
    LoggingIn,
    LoginTip,
    Tired,
    NeedReopen,
    MsgSending(u64),
    Talking,
}

pub struct ChatGPT {
    client: Client,
}

impl ChatGPT {
    pub async fn new() -> Self {
        // Define the Chrome capabilities
        let mut caps = serde_json::map::Map::new();
        caps.insert("goog:chromeOptions".to_string(),
                    json!({
                        "args": [
                            // "--headless",
                            // "--disable-gpu",
                            "--no-sandbox",
                            "--disable-dev-shm-usage",
                            "--disable-blink-features=AutomationControlled",
                            "--disable-features=InterestCohort",
                            "--disable-features=BrowsingTopics",
                            // "--proxy-server=127.0.0.1:7890",
                            // start a remote port for CDP protocol
                            "--remote-debugging-port=9222",
                            "user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                            "disable-infobars",
                        ],
                        "excludeSwitches": ["enable-automation"]
                    })
        );

        // start chromedriver at port 9515 before launching this program
        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect("http://localhost:9515")
            .await
            .expect("connect to http://localhost:9515");

        ChatGPT { client }
    }

    pub async fn new_session(&self, session_opened: Arc<AtomicBool>) {
        if let Err(error) = self.client.goto("https://chatgpt.com/").await {
            panic!("go to https://chatgpt.com: {error:#?}");
        }
        session_opened.store(true, Ordering::Release);
        loop {
            println!("try to login in...");
            if self.get_chatbox(5).await.is_none() && self.open_chatbox().await.is_none() {
                continue;
            }
            break;
        }
    }

    #[allow(unreachable_code)]
    pub async fn send_my_said(&self, said: &str) -> Result<(), fantoccini::error::CmdError> {
        loop {
            println!("1. get_chatbox");
            let mut chatbox = self.get_chatbox(5).await;
            if chatbox.is_none() {
                // util::pause();
                println!("2. open_chatbox");
                chatbox = self.open_chatbox().await;
                if chatbox.is_none() {
                    continue;
                }
            }

            while WebState::is_talking(&self.client).await {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            println!("mymsg is\n{}", said);
            println!("ready to send immediately...");
            let _ = util::pause().await;
            self.set_user_msg(said).await?;

            // wait until user the message could be sent to openai
            #[allow(unused_assignments)]
            let mut msg_sent = false;
            loop {
                println!("4. send_user_msg");
                if self.send_user_msg().await? {
                    println!("5. message sent..");
                    msg_sent = true;
                    break;
                } else {
                    println!("6. open_chatbox");
                    let _ = self.open_chatbox().await;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
            if msg_sent {
                break;
            }

            // take a break
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        Ok(())
    }

    async fn set_user_msg(&self, msg: &str) -> Result<(), fantoccini::error::CmdError> {
        let msg = msg.replace("\n", "\\n");
        let msg = msg.replace("\t", "\\t");
        let msg = msg.replace("'", "\\'");
        if let Err(error) = self
            .client
            .execute(
                &format!(
                    "document.querySelector('#prompt-textarea').value = '{}'",
                    msg
                ),
                vec![],
            )
            .await
        {
            println!("set_user_msg: {error:#?}");
        }
        Ok(())
    }

    async fn send_user_msg(&self) -> Result<bool, fantoccini::error::CmdError> {
        // make #prompt-textarea active else the send button will still be disabled
        if let Some(chatbox) = self.get_chatbox(1).await {
            if let Err(error) = chatbox.send_keys("\n").await {
                println!("send_keys: {error:#?}");
                util::pause_force().await;
                return Ok(false);
            }
        }

        // have to re-find the send button
        let send_btn = get_send_btn(&self.client).await;

        // println!("Are yre ready to click?");
        let _ = util::pause().await;

        println!("send button: {:#?}", send_btn.html(false).await);
        if let Err(error) = &send_btn.click().await {
            println!("send-button click: {error:#?}");
            Ok(false)
        } else {
            // at this time, send_btn could be staled
            println!("send button clicked");
            Ok(true)
        }
    }

    async fn open_chatbox(&self) -> Option<Element> {
        match WebState::get(&self.client).await {
            WebState::LoggingIn => {
                println!("1.1 logging in...");
                self.bypass_cloudfare().await.ok()
            }
            WebState::LoginTip => {
                println!("1.2 logging tip...");
                let _ = WebState::close_login_tip(&self.client).await;
                self.get_chatbox(1).await
            }
            WebState::Tired => {
                println!("1.3 tired...");
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                self.get_chatbox(1).await
            }
            WebState::NeedReopen => {
                println!("1.4 reopen...");
                let _ = WebState::reopen_chatbox(&self.client).await;
                let cb = self.get_chatbox(1).await;
                if cb.is_none() {
                    if let Err(error) = self.client.refresh().await {
                        println!("refresh page: {error:#?}");
                    }
                }
                cb
            }
            WebState::ChatReady => {
                println!("1.5 chat ready...");
                self.get_chatbox(1).await
            }
            WebState::MsgSending(ref mut n) => {
                println!("1.6 msg sending...{}", *n);
                if *n > 5 {
                    if let Err(error) = self.client.refresh().await {
                        println!("refresh page: {error:#?}");
                    }
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    *n = *n + 1;
                }
                self.get_chatbox(1).await
            }
            WebState::Talking => self.get_chatbox(1).await,
        }
    }

    async fn resort_to_rescue_page(&self) -> Result<(), fantoccini::error::CmdError> {
        let client = &self.client;

        client.goto("https://openai.com/index/chatgpt/").await?;

        // "Try ChatGPT" butthon
        let href_value = "https://chatgpt.com/";
        let css_selector = format!("a[href='{}']", href_value);
        let link_btn_selector = Locator::Css(&css_selector);
        client
            .wait()
            .at_most(std::time::Duration::from_secs(10))
            .for_element(link_btn_selector)
            .await?;
        let link_btn = client.find(link_btn_selector).await?;
        // wait until clickable
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        link_btn.click().await?;

        client
            .switch_to_window(client.windows().await?.remove(1))
            .await
    }

    async fn humankind_identify(&self) -> Result<(), fantoccini::error::CmdError> {
        self.client.enter_frame(Some(0)).await?;
        // the id could be an encrypted string
        // let button_selector = Locator::Css("#challenge-stage");
        // client.wait().at_most(std::time::Duration::from_secs(10)).for_element(button_selector).await?;
        // let button = client.find(button_selector).await?;

        // Find the cloudfare input checkbox.
        let checkbox_loc = Locator::Css("input[type=\"checkbox\"]");
        let rst = self
            .client
            .wait()
            .at_most(std::time::Duration::from_secs(10))
            .for_element(checkbox_loc)
            .await;
        if let Err(error) = rst {
            println!("input checkbox: {error:#?}");
            return Ok(());
        }
        let checkbox = rst.unwrap();
        println!("checkbox found");
        let button: Element;
        let mut path = "./..".to_string();
        loop {
            // find a parent element with an id
            let checkbox = checkbox.find(Locator::XPath(&path)).await?;
            if !checkbox.attr("id").await?.is_none() {
                button = checkbox;
                break;
            }
            println!("up: {:#?}", checkbox.tag_name().await?);
            path.push_str("/..");
            // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        button.click().await
    }

    async fn get_chatbox(&self, timeout: u64) -> Option<Element> {
        let elm = self
            .client
            .wait()
            .at_most(std::time::Duration::from_secs(timeout))
            .for_element(Locator::Css("#prompt-textarea"))
            .await;
        if let Err(error) = elm {
            println!("get #prompt-textarea: {:#?}", error);
            // util::pause_force().await;
            return None;
        }
        return elm.ok();
    }

    // Idea from https://github.com/ultrafunkamsterdam/undetected-chromedriver/issues/73#issuecomment-748487642
    #[allow(unreachable_code)]
    async fn bypass_cloudfare(&self) -> Result<Element, fantoccini::error::CmdError> {
        println!("Try to bypass cloudfare...");

        self.humankind_identify().await?;
        let checkbox = self.get_chatbox(1).await;
        if let Some(checkbox) = checkbox {
            return Ok(checkbox);
        }

        self.resort_to_rescue_page().await?;

        let client = &self.client;

        let checkbox = self.get_chatbox(1).await;
        if checkbox.is_none() {
            println!("Oooops! You need login manually to chatgpt.com");
            client
                .switch_to_window(client.windows().await?.remove(0))
                .await?;
        }
        loop {
            println!("Try to click the 'Try ChatGPT' button!");
            if client.windows().await?.len() > 2 {
                client
                    .switch_to_window(client.windows().await?.remove(2))
                    .await?;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        // switch to the first tab page
        client
            .switch_to_window(client.windows().await?.remove(0))
            .await?;

        // wait for the user to login the first tab page
        let mut n = 0;
        loop {
            let checkbox = self.get_chatbox(1).await;
            if checkbox.is_some() {
                break;
            }
            n += 1;
            println!("wait for user login...{n}");
        }
        println!("You just logged in! Have Fun!");

        Ok(self.get_chatbox(1).await.unwrap())
    }

    pub async fn close(self) -> Result<(), fantoccini::error::CmdError> {
        // Close the browser
        self.client.close().await
    }
}

async fn get_send_btn(client: &Client) -> Element {
    loop {
        let mut btn = client
            .wait()
            .at_most(std::time::Duration::from_secs(2))
            .for_element(Locator::Css("button[data-testid=\"send-button\"]"))
            .await;

        if btn.is_err() {
            let _ = util::pause().await;

            let btns = client.find_all(Locator::Css("button[data-testid]")).await;
            if let Err(_) = btns {
                println!("could not get any button[data-testid]...sleep a while...");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            let mut found = false;
            let mut index = 0;
            let mut btns = btns.unwrap();
            for (i, btn) in btns.iter().rev().enumerate() {
                if let Ok(Some(val)) = btn.attr("data-testid").await {
                    if val.contains("send-button") {
                        found = true;
                        index = i;
                        break;
                    }
                }
            }
            if !found {
                println!("could not get any button[data-testid=\"*send-button\"]");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            btn = Ok(btns.remove(index));
        }
        return btn.unwrap();
    }
}

impl WebState {
    async fn get(client: &Client) -> Self {
        if WebState::is_logging_in(client).await {
            WebState::LoggingIn
        } else if WebState::is_login_tip(client).await {
            WebState::LoginTip
        } else if WebState::is_tired(client).await {
            WebState::Tired
        } else if WebState::need_reopen(client).await {
            WebState::NeedReopen
        } else if WebState::is_msg_sending(client).await {
            WebState::MsgSending(0)
        } else {
            WebState::ChatReady
        }
    }

    async fn is_msg_sending(client: &Client) -> bool {
        let btn = client
            .find(Locator::Css("button[data-testid=\"send-button\"]"))
            .await
            .unwrap();
        !btn.is_enabled().await.unwrap() && btn.find(Locator::Css("svg.animate-spin")).await.is_ok()
    }

    async fn is_login_tip(client: &Client) -> bool {
        client
            .find(Locator::Css("div[role=\"dialog\"]"))
            .await
            .is_ok()
    }

    async fn close_login_tip(client: &Client) -> Result<(), fantoccini::error::CmdError> {
        let dialog = client
            .find(Locator::Css("div[role=\"dialog\"]"))
            .await
            .expect("not in LoginTip state");
        let link = dialog
            .find(Locator::Css("div > div > a"))
            .await
            .expect("保持注销状态");
        link.click().await
    }

    async fn is_tired(client: &Client) -> bool {
        match WebState::last_assistant_message(&client).await {
            Ok(opts) => {
                if opts.is_none() {
                    return false;
                }
                opts.unwrap()
                    == "You've reached our limit of messages per hour. Please try again later."
            }
            Err(_) => false,
        }
    }

    async fn last_assistant_message(
        client: &Client,
    ) -> Result<Option<String>, fantoccini::error::CmdError> {
        match client
            .find_all(Locator::Css("div[data-message-author-role=\"assistant\"]"))
            .await?
            .pop()
            .map(|element| async move { element.text().await })
        {
            Some(fut) => fut.await.map(|s| Some(s)),
            None => Ok(None),
        }
    }

    /// this function should not be invoked on WebState::LoginTip state
    async fn need_reopen(client: &Client) -> bool {
        debug_assert_eq!(WebState::is_login_tip(&client).await, false);
        client.find(Locator::Css("#prompt-textarea")).await.is_err()
    }

    /// find reversely the first button having text content "重新生成"
    async fn reopen_chatbox(client: &Client) -> Result<(), fantoccini::error::CmdError> {
        for btn in client
            .find_all(Locator::Css("button"))
            .await
            .unwrap()
            .iter()
            .rev()
        {
            if btn.text().await.unwrap() == "重新生成" {
                btn.click().await?
            }
        }
        Ok(())
    }

    async fn is_logging_in(client: &Client) -> bool {
        client.find(Locator::Css("#challenge-form")).await.is_ok()
    }

    async fn is_talking(client: &Client) -> bool {
        let btn = get_send_btn(client).await;
        btn.is_enabled().await.unwrap_or(false)
            && btn.find(Locator::Css("svg > rect")).await.is_ok()
    }
}
