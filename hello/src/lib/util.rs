use futures::{SinkExt, StreamExt};
use reqwest;
use serde_json::json;
use std::io::Write; // flush
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// Listen specific web page with `page_url` at the tab `index`
/// Before we invoke the function, lauch Chrome program with
/// '--remote-debugging-port=<browser_port>'.
pub async fn listen_webpage_stream_data(
    browser_port: u16,
    page_url: &str,
    index: usize,
    request_url: &str,
    handle_fn: impl Fn(&str),
) -> Result<(), Box<dyn std::error::Error>> {
    let response_text = reqwest::get(format!("http://localhost:{browser_port}/json"))
        .await?
        .text()
        .await?;
    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
    let mut url = String::new();
    for (i, page) in response_json.as_array().unwrap().iter().enumerate() {
        if index == i && page.get("url").unwrap().as_str().unwrap() == page_url {
            url = page
                .get("webSocketDebuggerUrl")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            break;
        }
    }

    println!("connect to page {url}");
    let (mut ws_stream, _) = connect_async(url).await?;

    // https://github.com/aslushnikov/getting-started-with-cdp/blob/master/README.md
    // Enable network monitoring
    let enable_network = json!({
        "id": next_command_id(),
        "method": "Network.enable"
    });
    ws_stream
        .send(Message::Text(enable_network.to_string()))
        .await?;

    let mut stream_res_cont_cid = 0;
    let mut conversation_request_id = String::new();

    println!("Listening for network events...");
    while let Some(msg) = ws_stream.next().await {
        // println!("{msg:#?}");
        match msg? {
            Message::Text(text) => {
                let json_msg: serde_json::Value = serde_json::from_str(&text)?;
                if let Some(method) = json_msg["method"].as_str() {
                    match method {
                        "Network.responseReceived" => {
                            let params = &json_msg["params"];
                            let request_id = params["requestId"].as_str().unwrap_or("0");
                            // let wall_time = params["wallTime"].as_str().unwrap_or("0");
                            let url = params["response"]["url"].as_str().unwrap_or("0");

                            if url == request_url {
                                // println!(
                                //     "requestId: {}, wallTime: {}, response.url: {}",
                                //     request_id, wall_time, url
                                // );

                                // https://chromedevtools.github.io/devtools-protocol/tot/Network/#method-streamResourceContent
                                stream_res_cont_cid = next_command_id();
                                conversation_request_id = request_id.to_string();
                                let enable_stream_data = json!({
                                    "id": stream_res_cont_cid,
                                    "method": "Network.streamResourceContent",
                                    "params": {
                                        "requestId": request_id
                                    }
                                });
                                // println!(
                                //     "enable streaming resource content for request {request_id}"
                                // );
                                ws_stream
                                    .send(Message::Text(enable_stream_data.to_string()))
                                    .await?;
                            }
                        }
                        // https://chromedevtools.github.io/devtools-protocol/tot/Network/#event-dataReceived
                        "Network.dataReceived" => {
                            // protocol event
                            if json_msg.get("id").is_none() {
                                let request_id = json_msg["params"]["requestId"].as_str();
                                if request_id.unwrap_or("") == conversation_request_id {
                                    let data = json_msg["params"]["data"].as_str();
                                    let data = decode_base64(data.unwrap_or(""))?;
                                    handle_fn(&data);
                                }
                            } else {
                                // command response
                                let msgid = json_msg["id"].as_number().unwrap().as_u64().unwrap();
                                if msgid == stream_res_cont_cid {
                                    let databuf = &json_msg["result"]["bufferedData"].as_str();
                                    let data = decode_base64(databuf.unwrap_or(""))?;
                                    handle_fn(&data);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn next_command_id() -> u64 {
    static SEQ: AtomicUsize = AtomicUsize::new(1);
    SEQ.fetch_add(1, Ordering::Relaxed) as u64
}

fn decode_base64(encoded: &str) -> Result<String, base64::DecodeError> {
    use base64::{engine::general_purpose, Engine};

    // Decode the base64 string to bytes
    let decoded_bytes = general_purpose::STANDARD.decode(encoded)?;

    // Convert the bytes to a String
    let decoded_string = String::from_utf8(decoded_bytes)
        .expect("Failed to convert bytes to string")
        .to_string();

    Ok(decoded_string)
}

pub fn pause_sync() -> String {
    if let Ok(pause_var) = std::env::var("PAUSE") {
        if pause_var == "1" {
            println!("Press Enter to continue...");
            std::io::stdout().flush().unwrap(); // Ensure the message is displayed immediately
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            return input;
        }
    }
    String::from("")
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn pause() -> String {
    if let Ok(pause_var) = std::env::var("PAUSE") {
        if pause_var == "1" {
            let mut stdout = tokio::io::stdout();
            stdout
                .write_all(b"Press Enter to continue...\n")
                .await
                .unwrap();
            stdout.flush().await.unwrap(); // Ensure the message is displayed immediately

            let mut stdin = BufReader::new(tokio::io::stdin());
            let mut input = String::new();
            stdin.read_line(&mut input).await.unwrap();
            return input;
        } else if pause_var == "2" {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            return "continue".to_string();
        }
    }
    String::from("")
}

pub async fn pause_force() -> String {
    let mut stdout = tokio::io::stdout();
    stdout
        .write_all(b"Emergency!!! Press Enter to continue...\n")
        .await
        .unwrap();
    stdout.flush().await.unwrap(); // Ensure the message is displayed immediately

    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut input = String::new();
    stdin.read_line(&mut input).await.unwrap();
    return input;
}
