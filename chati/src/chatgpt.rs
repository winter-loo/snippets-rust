#![allow(dead_code)]

use fantoccini::actions::{InputSource, MouseActions, PointerAction, MOUSE_BUTTON_LEFT};
use fantoccini::{client::*, elements::*, ClientBuilder, Locator};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio;
use log::{debug, info, error};

use crate::util;

enum WebState {
    ChatReady,
    LoggingIn,
    LoginTip,
    Tired,
    NeedReopen,
    /// send button is spinning
    MsgSending,
    Talking,
}

pub struct ChatGPT {
    client: Client,
    /// when the page is stucking in sending user message, we need remember how long have we waited
    /// for. If the wait time exceeds a limit, we need refresh page.
    sending_sleep: u64,
    // /// When we refresh the page, we need send these initial prompts to tell chatgpt what we want
    // /// do. Responses(or assistant messages) will be discarded.
    // initial_prompts: Vec<String>,
}

impl ChatGPT {
    pub async fn new(/* initial_prompts: Vec<String>*/) -> Self {
        let mut proxy_server = std::env::var("http_proxy").unwrap_or("".to_string());
        let mut args = json!({
            "args": [
                // "--headless",
                // "--disable-gpu",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-blink-features=AutomationControlled",
                "--disable-features=InterestCohort",
                "--disable-features=BrowsingTopics",
                // start a remote port for CDP protocol
                "--remote-debugging-port=9222",
                "user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                "disable-infobars",
            ],
            "excludeSwitches": ["enable-automation"]
        });
        if !proxy_server.is_empty() {
            if proxy_server.starts_with("http://") {
                proxy_server = proxy_server[7..].to_string();
            }
            args["args"].as_array_mut().unwrap().push(json! {
                format!("--proxy-server={}", proxy_server)
            });
        }
        info!("args: {:#?}", args);
        // Define the Chrome capabilities
        let mut caps = serde_json::map::Map::new();
        caps.insert("goog:chromeOptions".to_string(), args);

        // start chromedriver at port 9515 before launching this program
        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect("http://localhost:9515")
            .await
            .expect("connect to http://localhost:9515");

        ChatGPT {
            client,
            sending_sleep: 0,
            // initial_prompts,
        }
    }

    pub async fn new_session(&mut self, session_opened: Arc<AtomicBool>) {
        if let Err(error) = self.client.goto("https://chatgpt.com/").await {
            panic!("go to https://chatgpt.com: {error:#?}");
        }
        session_opened.store(true, Ordering::Release);
        loop {
            debug!("try to login in...");
            if self.get_chatbox(5).await.is_none() && self.open_chatbox().await.is_none() {
                continue;
            }
            break;
        }
    }

    pub async fn wait_for_chatbox(&self, session_opened: Arc<AtomicBool>) {
        if let Err(error) = self.client.goto("https://chatgpt.com/").await {
            panic!("go to https://chatgpt.com: {error:#?}");
        }
        session_opened.store(true, Ordering::Release);
        loop {
            if self.get_chatbox(1).await.is_some() {
                break;
            }
            debug!("waiting for chatbox available...");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    #[allow(unreachable_code)]
    pub async fn send_my_said(&mut self, said: &str) {
        loop {
            debug!("1. get_chatbox");
            let mut chatbox = self.get_chatbox(5).await;
            if chatbox.is_none() {
                // util::pause();
                debug!("2. open_chatbox");
                chatbox = self.open_chatbox().await;
                if chatbox.is_none() {
                    continue;
                }
            }

            while WebState::is_talking(&self.client).await {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // wait until user the message could be sent to openai
            #[allow(unused_assignments)]
            let mut msg_sent = false;
            // `set_user_msg` and `send_user_msg` bost must be in the same loop
            loop {
                debug!("ready to set user message immediately...");
                let _ = util::pause().await;
                self.set_user_msg(said).await;

                debug!("4. send_user_msg");
                if self.send_user_msg().await {
                    debug!("5. message sent..");
                    msg_sent = true;
                    break;
                } else {
                    debug!("6. open_chatbox");
                    let _ = self.open_chatbox().await;
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
            if msg_sent {
                break;
            }

            // take a break
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    async fn set_user_msg(&self, msg: &str) {
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
            debug!("set_user_msg: {error:#?}");
        }
    }

    async fn send_user_msg(&self) -> bool {
        // make #prompt-textarea active else the send button will still be disabled
        if let Some(chatbox) = self.get_chatbox(1).await {
            if let Err(error) = chatbox.send_keys(" ").await {
                error!("send_keys: {error:#?}");
                // util::pause_force().await;
                return false;
            }
        }

        // have to re-find the send button
        let send_btn = match get_send_btn(&self.client).await {
            Some(send_btn) => send_btn,
            None => return false,
        };
        if !send_btn.is_enabled().await.unwrap() {
            return false;
        }

        // println!("Are yre ready to click?");
        let _ = util::pause().await;

        let mouse_click = MouseActions::new("click send button".to_string())
            .then(PointerAction::MoveToElement {
                element: send_btn,
                duration: Some(std::time::Duration::from_millis(200)),
                x: 1,
                y: 1,
            })
            .then(PointerAction::Down {
                button: MOUSE_BUTTON_LEFT,
            })
            .then(PointerAction::Up {
                button: MOUSE_BUTTON_LEFT,
            });
        if let Err(error) = self.client.perform_actions(mouse_click).await {
            error!("error on mouse click the send button: {error:#?}");
            false
        } else {
            true
        }
    }

    // TODO: refreshing a page means we need do more things.
    // Such as,
    //  * resend initial command prompts
    async fn restart_session(&mut self, doit: bool) {
        if doit {
            if let Err(error) = self.client.refresh().await {
                panic!("refresh page: {error:#?}");
            }
        }
    }

    async fn open_chatbox(&mut self) -> Option<Element> {
        let webstate = WebState::get(&self.client).await;
        match webstate {
            WebState::MsgSending => {
                debug!("1.6 msg sending...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                self.sending_sleep += 1;
                if self.sending_sleep > 10 {
                    self.sending_sleep = 0;
                    self.restart_session(false).await;
                }
            }
            _ => {
                self.sending_sleep = 0;
            }
        }
        match webstate {
            WebState::LoggingIn => {
                debug!("1.1 logging in...");
                self.bypass_cloudfare().await.ok()
            }
            WebState::LoginTip => {
                debug!("1.2 logging tip...");
                let _ = WebState::close_login_tip(&self.client).await;
                self.get_chatbox(1).await
            }
            WebState::Tired => {
                debug!("1.3 tired, sleep 10 minutes...");
                tokio::time::sleep(std::time::Duration::from_secs(600)).await;
                self.get_chatbox(1).await
            }
            WebState::NeedReopen => {
                debug!("1.4 reopen...");
                let _ = WebState::reopen_chatbox(&self.client).await;
                let cb = self.get_chatbox(1).await;
                if cb.is_none() {
                    self.restart_session(true).await;
                }
                cb
            }
            WebState::ChatReady => {
                debug!("1.5 chat ready...");
                self.get_chatbox(1).await
            }
            WebState::Talking => self.get_chatbox(1).await,
            _ => None,
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
            error!("input checkbox: {error:#?}");
            return Ok(());
        }
        let checkbox = rst.unwrap();
        debug!("checkbox found");
        let button: Element;
        let mut path = "./..".to_string();
        loop {
            // find a parent element with an id
            let checkbox = checkbox.find(Locator::XPath(&path)).await?;
            if !checkbox.attr("id").await?.is_none() {
                button = checkbox;
                break;
            }
            debug!("up: {:#?}", checkbox.tag_name().await?);
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
            error!("get #prompt-textarea: {:#?}", error);
            use fantoccini::error::CmdError;
            match error {
                CmdError::Lost(_) => {
                    std::process::exit(0);
                }
                CmdError::NoSuchWindow(_) => {
                    std::process::exit(0);
                }
                _ => {}
            }
            // util::pause_force().await;
            return None;
        }
        return elm.ok();
    }

    // Idea from https://github.com/ultrafunkamsterdam/undetected-chromedriver/issues/73#issuecomment-748487642
    #[allow(unreachable_code)]
    async fn bypass_cloudfare(&self) -> Result<Element, fantoccini::error::CmdError> {
        debug!("Try to bypass cloudfare...");

        self.humankind_identify().await?;
        let checkbox = self.get_chatbox(1).await;
        if let Some(checkbox) = checkbox {
            return Ok(checkbox);
        }

        self.resort_to_rescue_page().await?;

        let client = &self.client;

        let checkbox = self.get_chatbox(1).await;
        if checkbox.is_none() {
            info!("Oooops! You need login manually to chatgpt.com");
            client
                .switch_to_window(client.windows().await?.remove(0))
                .await?;
        }
        loop {
            info!("Try to click the 'Try ChatGPT' button!");
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
            info!("wait for user login...{n}");
        }
        info!("You just logged in! Have Fun!");

        Ok(self.get_chatbox(1).await.unwrap())
    }

    pub async fn close(self) -> Result<(), fantoccini::error::CmdError> {
        // Close the browser
        self.client.close().await
    }
}

async fn get_send_btn(client: &Client) -> Option<Element> {
    let mut btn = client
        .wait()
        .at_most(std::time::Duration::from_secs(2))
        .for_element(Locator::Css("button[data-testid=\"send-button\"]"))
        .await;

    if btn.is_err() {
        let btns = client.find_all(Locator::Css("button[data-testid]")).await;
        if let Err(_) = btns {
            error!("could not get any button[data-testid]...sleep a while...");
            return None;
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
            error!("could not get any button[data-testid=\"*send-button\"]");
            return None;
        }
        btn = Ok(btns.remove(btns.len() - 1 - index));
    }
    return Some(btn.unwrap());
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
            WebState::MsgSending
        } else {
            WebState::ChatReady
        }
    }

    async fn is_msg_sending(client: &Client) -> bool {
        let btn = get_send_btn(client).await;
        match btn {
            Some(btn) => {
                !btn.is_enabled().await.unwrap()
                    && btn.find(Locator::Css("svg.animate-spin")).await.is_ok()
            }
            None => false,
        }
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
        match btn {
            Some(btn) => {
                btn.is_enabled().await.unwrap_or(false)
                    && btn.find(Locator::Css("svg > rect")).await.is_ok()
            }
            None => false,
        }
    }
}
