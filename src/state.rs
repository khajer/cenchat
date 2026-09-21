use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

#[derive(Default)]
pub struct Room {
    members: HashMap<SocketAddr, (String, UnboundedSender<Message>)>,
}

#[derive(Default)]
pub struct SharedState {
    rooms: HashMap<String, Room>,
}

pub type Shared = Arc<Mutex<SharedState>>;

impl SharedState {
    /// Adds `addr` to `room` (creating it if absent). Returns the room's member count after joining.
    pub fn join(
        &mut self,
        room: &str,
        addr: SocketAddr,
        name: &str,
        tx: UnboundedSender<Message>,
    ) -> usize {
        let room = self.rooms.entry(room.to_string()).or_default();
        room.members.insert(addr, (name.to_string(), tx));
        room.members.len()
    }

    /// Removes `addr` from `room`, garbage-collecting the room if it becomes empty.
    pub fn leave(&mut self, room: &str, addr: &SocketAddr) {
        if let Some(r) = self.rooms.get_mut(room) {
            r.members.remove(addr);
            if r.members.is_empty() {
                self.rooms.remove(room);
            }
        }
    }

    /// Sends `msg` to every member currently in `room`.
    pub fn broadcast(&self, room: &str, msg: &Message) {
        if let Some(r) = self.rooms.get(room) {
            for (_, tx) in r.members.values() {
                let _ = tx.send(msg.clone());
            }
        }
    }

    /// Returns the sorted usernames of everyone currently in `room`.
    pub fn user_names(&self, room: &str) -> Vec<String> {
        let mut names: Vec<String> = self
            .rooms
            .get(room)
            .map(|r| r.members.values().map(|(name, _)| name.clone()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}
