use connor::ConnorServer;
use std::process::exit;

#[tokio::main]
async fn main() {
    let connor_server = ConnorServer::new();
    if let Err(err) = connor_server.start().await {
        println!("{:?}", err);
        exit(1);
    }
}
