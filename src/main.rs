mod connection;
mod protocol;
mod state;

use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use state::Shared;

const ADDR: &str = "127.0.0.1:9001";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let listener = TcpListener::bind(ADDR)
        .await
        .expect("failed to bind websocket listener");
    info!("websocket server listening on ws://{ADDR}");

    let shared = Shared::default();

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("failed to accept connection: {err}");
                continue;
            }
        };

        let shared = shared.clone();
        tokio::spawn(async move {
            connection::handle_connection(stream, peer_addr, shared).await;
        });
    }
}
