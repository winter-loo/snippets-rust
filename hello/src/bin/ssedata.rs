use chromiumoxide::browser::{Browser, BrowserConfigBuilder};
use chromiumoxide::cdp::browser_protocol::network;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("connecting...");
    let (mut browser, mut handler) = Browser::connect("http://10.188.143.47:9222").await?;
    println!("connected...");
    let handle = tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                eprintln!("Error: {:?}", e);
            }
        }
    });

    println!("new page...");
    let page = browser.new_page("https://chatgpt.com").await?;
    println!("new page......");

    // Enable network event capturing
    page.execute(network::EnableParams {
        max_total_buffer_size: Some(1024 * 1024),
        max_resource_buffer_size: Some(1024 * 1024),
        max_post_data_size: Some(1024 * 1024),
    }).await?;

    // Listen for EventSource messages
    let mut events = page.event_listener::<network::EventEventSourceMessageReceived>().await?;
    
    while let Some(event) = events.next().await {
        println!("EventSource message: {:?}", event);
    }

    browser.close().await?;
    handle.await?;

    Ok(())
}
