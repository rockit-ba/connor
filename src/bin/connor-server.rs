//! 启动bin

use connor::server_bootstrap::ConnorServer;
use std::process::exit;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // 控制台打印
    tracing_subscriber::registry()
        .with(LevelFilter::from_level(Level::INFO))
        .with(fmt::layer())
        .init();

    let connor_server = &mut ConnorServer::new();
    if let Err(err) = connor_server.start().await {
        println!("{:?}", err);
        exit(1);
    }
}
