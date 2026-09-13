use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

const ADDR: &str = "127.0.0.1:9001";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(ADDR)
        .await
        .expect("failed to bind websocket listener");
    println!("websocket server listening on ws://{ADDR}");

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("failed to accept connection: {err}");
                continue;
            }
        };

        tokio::spawn(async move {
            handle_connection(stream, peer_addr).await;
        });
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, peer_addr: std::net::SocketAddr) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            eprintln!("[{peer_addr}] websocket handshake failed: {err}");
            return;
        }
    };
    println!("[{peer_addr}] connected");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("[{peer_addr}] error reading message: {err}");
                break;
            }
        };

        match msg {
            Message::Text(_) | Message::Binary(_) => {
                if let Err(err) = write.send(msg).await {
                    eprintln!("[{peer_addr}] error echoing message: {err}");
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    println!("[{peer_addr}] disconnected");
}
