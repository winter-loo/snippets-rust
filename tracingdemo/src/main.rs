use tracing::{error, event, info, trace, Level};

fn main() {
    // use tracing_subscriber to output tracing messages
    tracing_subscriber::fmt()
    // allow to use environment variables to configure tracing(RUST_LOG=trace)
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    // .with_max_level(Level::TRACE)
    .init();

    info!("Hello, world!");
    event!(Level::INFO, "moment!");
    trace!("trace");
    error!("error");
    println!("ok");
}
