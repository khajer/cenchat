use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

use crate::protocol::{self, Command};
use crate::state::Shared;

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, shared: Shared) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(err) => {
            error!("[{addr}] websocket handshake failed: {err}");
            return;
        }
    };
    info!("[{addr}] connected");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut name: Option<String> = None;
    let mut room: Option<String> = None;

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                error!("[{addr}] error reading message: {err}");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                dispatch(&shared, &tx, addr, &mut name, &mut room, protocol::parse_line(&text));
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    if let Some(room_name) = &room {
        leave_room(&shared, &tx, addr, &name, room_name);
    }

    drop(tx);
    let _ = writer_task.await;
    info!("[{addr}] disconnected");
}

fn dispatch(
    shared: &Shared,
    tx: &mpsc::UnboundedSender<Message>,
    addr: SocketAddr,
    name: &mut Option<String>,
    room: &mut Option<String>,
    command: Command,
) {
    match command {
        Command::Name(new_name) => {
            if new_name.is_empty() || new_name.contains(char::is_whitespace) {
                let _ = tx.send(protocol::err("usage: /name <name>"));
                return;
            }
            *name = Some(new_name.clone());
            let _ = tx.send(protocol::sys(format!("name set to {new_name}")));
        }
        Command::Join(room_name) => {
            let Some(current_name) = name.as_ref() else {
                let _ = tx.send(protocol::err("set your name first with /name <name>"));
                return;
            };
            if room_name.is_empty() || room_name.contains(char::is_whitespace) {
                let _ = tx.send(protocol::err("usage: /join <room>"));
                return;
            }

            if let Some(old_room) = room.take() {
                leave_room(shared, tx, addr, name, &old_room);
            }

            let count = {
                let mut state = shared.lock().unwrap();
                let count = state.join(&room_name, addr, current_name, tx.clone());
                state.broadcast(&room_name, &protocol::sys(format!("{current_name} joined the room")));
                count
            };
            *room = Some(room_name.clone());
            let _ = tx.send(protocol::sys(format!("joined room {room_name} ({count} users)")));
        }
        Command::Leave => {
            let Some(room_name) = room.take() else {
                let _ = tx.send(protocol::err("you are not in a room"));
                return;
            };
            leave_room(shared, tx, addr, name, &room_name);
            let _ = tx.send(protocol::sys(format!("left room {room_name}")));
        }
        Command::Who => {
            let Some(room_name) = room.as_ref() else {
                let _ = tx.send(protocol::err("join a room first with /join <room>"));
                return;
            };
            let names = shared.lock().unwrap().user_names(room_name);
            let _ = tx.send(protocol::users_list(&names));
        }
        Command::Text(text) => {
            let Some(current_name) = name.as_ref() else {
                let _ = tx.send(protocol::err("set your name first with /name <name>"));
                return;
            };
            let Some(room_name) = room.as_ref() else {
                let _ = tx.send(protocol::err("join a room first with /join <room>"));
                return;
            };
            shared
                .lock()
                .unwrap()
                .broadcast(room_name, &protocol::chat(current_name, &text));
        }
        Command::Unknown(cmd) => {
            let _ = tx.send(protocol::err(format!("unknown command: /{cmd}")));
        }
    }
}

fn leave_room(
    shared: &Shared,
    _tx: &mpsc::UnboundedSender<Message>,
    addr: SocketAddr,
    name: &Option<String>,
    room_name: &str,
) {
    let mut state = shared.lock().unwrap();
    state.leave(room_name, &addr);
    if let Some(current_name) = name {
        state.broadcast(room_name, &protocol::sys(format!("{current_name} left the room")));
    }
}
