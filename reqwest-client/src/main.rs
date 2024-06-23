use reqwest;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ns() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("time went backwards");
    since_epoch.as_secs() * 1_000_000_000 + since_epoch.subsec_nanos() as u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} send request", now_ns());
    let client = reqwest::blocking::ClientBuilder::new().timeout(None).build()?;
    let response = client.get("http://localhost:3000/hello/ldd").send()?;
    if response.status().is_success() {
        let body = response.text()?;
        println!("{} Response: {}", body, now_ns());
    }
    Ok(())
}
