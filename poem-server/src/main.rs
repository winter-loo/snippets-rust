use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::time::{SystemTime, UNIX_EPOCH};
use console_subscriber;
use poem::{get, handler, listener::TcpListener, web::Path, Route, Server};

lazy_static! {
    static ref COUNT: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

fn now_ns() -> u64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("time went backwards");
    since_epoch.as_secs() * 1_000_000_000 + since_epoch.subsec_nanos() as u64
}

#[handler]
async fn hello(Path(name): Path<String>) -> String {
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    let mut count = COUNT.lock().unwrap();
    *count += 1;
    let ans = format!("{} hello({}): {}", now_ns(), count, name);
    ans
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    console_subscriber::init(); 
    let app = Route::new().at("/hello/:name", get(hello));
    Server::new(TcpListener::bind("0.0.0.0:3000"))
      .run(app)
      .await
}
