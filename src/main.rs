mod connection;
mod protocol;
mod state;

use tokio::net::TcpListener;

use state::Shared;

const ADDR: &str = "127.0.0.1:9001";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR)
        .await
        .expect("failed to bind websocket listener");
    println!("websocket server listening on ws://{ADDR}");

    let shared = Shared::default();

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("failed to accept connection: {err}");
                continue;
            }
        };

        let shared = shared.clone();
        tokio::spawn(async move {
            connection::handle_connection(stream, peer_addr, shared).await;
        });
    }
}
