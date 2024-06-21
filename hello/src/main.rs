use fantoccini::{ClientBuilder, Locator,  elements::*, client::*};
use tokio;
use serde_json::json;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
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
                        "--remote-debugging-port=9222",
                        "user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                        "disable-infobars",
                    ],
                    "excludeSwitches": ["enable-automation"]
                })
    );

    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://10.188.143.47:9515")
        .await.expect("failed to connect to WebDriver");

    
    client.goto("https://openai.com/index/chatgpt/").await?;

    // "Try ChatGPT" butthon
    let href_value = "https://chatgpt.com/";
    let css_selector = format!("a[href='{}']", href_value);
    let link_btn_selector = Locator::Css(&css_selector);
    client.wait().at_most(std::time::Duration::from_secs(10)).for_element(link_btn_selector).await?;
    let link_btn = client.find(link_btn_selector).await?;
    // wait until clickable
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    link_btn.click().await?;
    client.switch_to_window(client.windows().await?.remove(1)).await?;

    let chat_input_selector = Locator::Css("#prompt-textarea");
    loop {
        let rst = client.wait().at_most(std::time::Duration::from_secs(5)).for_element(chat_input_selector).await;
        if rst.is_err() {
            bypass_cloudfare(&client).await?;
        }
        let rst = client.find(chat_input_selector).await;
        let chatbox = rst.unwrap();

        if chatbox.send_keys("hello").await.is_err() {
            if WebState::is_login_tip(&client).await {
                let _ = WebState::enter_next_state(&client).await;
            } else if WebState::is_tired(&client).await {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                let _ = WebState::open_chat_channel(&client).await;
            } else if WebState::chat_channel_closed(&client).await {
                let _ = WebState::open_chat_channel(&client).await;
            }
            continue;
        }

        loop {
            // have to re-find the send button
            let send_elm = client.find(Locator::Css("button[data-testid=\"send-button\"]")).await;
            if ! send_elm.is_err() {
                if send_elm.unwrap().click().await.is_ok() {
                    println!("message sent");
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    // Close the browser
    client.close().await
}



// Idea from https://github.com/ultrafunkamsterdam/undetected-chromedriver/issues/73#issuecomment-748487642
async fn bypass_cloudfare(client: &fantoccini::Client) -> Result<(), fantoccini::error::CmdError> {
    println!("Try to bypass cloudfare...");

    client.enter_frame(Some(0)).await?;
    // the id could be an encrypted string
    // let button_selector = Locator::Css("#challenge-stage");
    // client.wait().at_most(std::time::Duration::from_secs(10)).for_element(button_selector).await?;
    // let button = client.find(button_selector).await?;

    let checkbox_loc = Locator::Css("input[type=\"checkbox\"]");
    let rst = client.wait().at_most(std::time::Duration::from_secs(10)).for_element(checkbox_loc).await;
    if rst.is_err() {
        panic!("Impossible! I can not find any checkbox in the page");
    }
    let checkbox = rst.unwrap();
    println!("checkbox found");
    let button: Element;
    let mut path = "./..".to_string();
    loop {
        // find a parent element with an id
        let checkbox = checkbox.find(Locator::XPath(&path)).await?;
        if ! checkbox.attr("id").await?.is_none() {
            button = checkbox;
            break;
        }
        println!("up: {:#?}", checkbox.tag_name().await?);
        path.push_str("/..");
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    button.click().await?;

    let chat_input_selector = Locator::Css("#prompt-textarea");
    let rst = client.wait().at_most(std::time::Duration::from_secs(5)).for_element(chat_input_selector).await;
    if rst.is_err() {
        println!("Oooops! You need manually login to chatgpt.com");
        client.switch_to_window(client.windows().await?.remove(0)).await?;
    }
    loop {
        println!("Try to click the 'Try ChatGPT' button!");
        if client.windows().await?.len() > 2 {
            client.switch_to_window(client.windows().await?.remove(2)).await?;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    let mut n = 0;
    loop {
        let rst = client.wait().at_most(std::time::Duration::from_secs(1)).for_element(chat_input_selector).await;
        if rst.is_ok() {
            println!("You logged in! Have a good day!");
            break;
        }
        n += 1;
        println!("wait for user login...{n}");
    }
    Ok(())
}

enum WebState {
    LoggingIn,
    ChatReady,
    LoginTip,
    Tired,
    Error,
}


struct ChatGPT<I, O>
where 
    I: Fn() -> String,
    O: Fn(&str) 
{
    client: Client,
    user_msg: I,
    asis_msg: O,
}

impl<I, O> ChatGPT<I, O>
where 
    I: Fn() -> String,
    O: Fn(&str) 
{
    async fn new(user: I, assistant: O) -> Self {
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
                            "user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
                            "disable-infobars",
                        ],
                        "excludeSwitches": ["enable-automation"]
                    })
        );

        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect("http://10.188.143.47:9515")
            .await.expect("failed to connect to WebDriver");

        ChatGPT {
            client,
            user_msg: user,
            asis_msg: assistant,
        }
    }

    fn startup() {

    }

    fn get_stat() -> WebState {
        WebState::LoggingIn
    }
}

impl WebState {
    async fn is_login_tip(client: &Client) -> bool {
        client.find(Locator::Css("div[role=\"dialog\"]")).await.is_ok()
    }

    async fn enter_next_state(client: &Client) -> Result<(), fantoccini::error::CmdError> {
        let dialog = client.find(Locator::Css("div[role=\"dialog\"]")).await.expect("not in LoginTip state");
        let link = dialog.find(Locator::Css("div > div > a")).await.expect("保持注销状态");
        link.click().await
    }

    async fn is_tired(client: &Client) -> bool {
        match WebState::last_assistant_message(&client).await {
            Ok(opts) => {
                if opts.is_none() {
                    return false;
                }
                opts.unwrap() == "You've reached our limit of messages per hour. Please try again later."
            },
            Err(_) => false,
        }
    }

    async fn last_assistant_message(client: &Client) -> Result<Option<String>, fantoccini::error::CmdError> {
        match client
            .find_all(Locator::Css("div[data-message-author-role=\"assistant\"]"))
            .await?
            .pop()
            .map(|element| async move { element.text().await }) {
            Some(fut) => {
                fut.await.map(|s| Some(s))
            },
            None => Ok(None),
        }
    }

    /// this function should not be invoked on WebState::LoginTip state
    async fn chat_channel_closed(client: &Client) -> bool {
        debug_assert_eq!(WebState::is_login_tip(&client).await, false);
        client.find(Locator::Css("#prompt-textarea")).await.is_err()
    }

    /// find reversely the first button having text content "重新生成"
    async fn open_chat_channel(client: &Client) -> Result<(), fantoccini::error::CmdError> {
        for btn in client.find_all(Locator::Css("button")).await.unwrap().iter().rev() {
            if btn.text().await.unwrap() == "重新生成" {
                btn.click().await?
            }
        }
        Ok(())
    }

    async fn is_logging_in(client: &Client) -> bool {
        false
    }

    async fn is_chatting(client: &Client) -> bool {
        false
    }

    async fn is_error(client: &Client) -> bool {
        false
    }
}
