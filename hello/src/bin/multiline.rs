use fantoccini::{ClientBuilder, Locator};

use hello::CommentExtractor;

use chromiumoxide::cdp::js_protocol::runtime::CallFunctionOnParams;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Run the async main function with tokio runtime
    // Create a browser instance
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head() // Optional: Run in headless mode or not
            .build()
            .unwrap(),
    )
    .await
    .expect("Failed to create browser instance");

    println!("Spawn the handler to run in the background");
    tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if let Err(e) = h {
                eprintln!("Error: {:?}", e);
            }
        }
    });

    println!("Create a new browser tab");
    let tab = browser
        .new_page("http://localhost:8000/foo.html")
        .await
        .unwrap();

    println!("Wait for the page to load");

    let infile = std::fs::File::open("bootstrap.c").expect("open file");
    let mut ce = CommentExtractor::new(infile);
    let cc = ce.next().expect("next comment");

    println!("Send keys (text) to the textarea element");
    // textarea.click().await.unwrap().type_str(&cc.content).await.unwrap();
    // tab.evaluate_expression("document.querySelector('textarea').value = 'hello\\nworld\\n1\\n2'").await.unwrap();
    let s = cc.content;
    let s = s.replace("\n", "\\n");
    let s = s.replace("\t", "\\t");
    let s = s.replace("'", "\\'");
    tab.evaluate_expression(format!(
        "document.querySelector('textarea').value = '{}'",
        s
    ))
    .await
    .unwrap();

    // Pause briefly to see the result (optional)
    tokio::time::sleep(std::time::Duration::from_secs(500)).await;

    // Close the browser
    drop(browser); // Close the browser explicitly
}

#[tokio::main]
async fn main2() {
    let client = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await
        .expect("Failed to connect to WebDriver");

    client.goto("http://localhost:8000/foo.html").await.unwrap();

    let s = "hello\\nworld\\n1\\n2\\t3\\t4\\t5";
    client
        .execute(
            &format!("document.querySelector('textarea').value = '{}'", s),
            vec![],
        )
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    client.close().await.unwrap();
}
