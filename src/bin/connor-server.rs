//! 启动bin

use connor::server_bootstrap::ConnorServer;
use std::process::exit;
use time::macros::format_description;
use tracing_subscriber::fmt::time::LocalTime;

#[tokio::main]
async fn main() {
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]-[minute]-[second]"
    ));
    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_line_number(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    let connor_server = &mut ConnorServer::new();
    if let Err(err) = connor_server.start().await {
        println!("{:?}", err);
        exit(1);
    }
}
