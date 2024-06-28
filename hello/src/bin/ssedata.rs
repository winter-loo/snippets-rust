use std::io::Write;

use chati::openai;
use chati::util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    util::listen_webpage_stream_data(
        9222,
        "https://chatgpt.com/",
        0,
        "https://chatgpt.com/backend-anon/conversation",
        |data| {
            openai::assistant_sse(data, |stream_msg, ended| {
                print!("{}", stream_msg);
                if let Err(e) = std::io::stdout().flush() {
                    eprintln!("Failed to flush stdout: {}", e);
                }
                if ended {
                    println!();
                }
            })
        },
    )
    .await?;
    Ok(())
}
