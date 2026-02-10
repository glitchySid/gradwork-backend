use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::chat::protocol::ServerMessage;

/// A handle to send messages to a connected WebSocket client.
#[derive(Debug, Clone)]
pub struct ClientHandle {
    pub user_id: Uuid,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

/// Manages all active WebSocket connections, organized by contract_id (chat room).
///
/// Each contract maps to a list of connected client handles. This allows
/// broadcasting messages, typing indicators, and presence updates to all
/// participants in a contract chat.
pub struct ChatServer {
    /// contract_id -> list of connected client handles
    rooms: RwLock<HashMap<Uuid, Vec<ClientHandle>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new WebSocket connection for a contract.
    /// Returns a receiver that the WebSocket session should listen on.
    pub async fn join(
        &self,
        contract_id: Uuid,
        user_id: Uuid,
    ) -> mpsc::UnboundedReceiver<ServerMessage> {
        let (tx, rx) = mpsc::unbounded_channel();

        let handle = ClientHandle {
            user_id,
            sender: tx,
        };

        // Notify existing participants that this user came online.
        let presence_msg = ServerMessage::Presence {
            user_id,
            online: true,
        };

        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(contract_id).or_insert_with(Vec::new);

        // Send presence to existing members before adding the new one.
        for client in room.iter() {
            if client.user_id != user_id {
                let _ = client.sender.send(presence_msg.clone());
            }
        }

        room.push(handle);

        rx
    }

    /// Remove a WebSocket connection for a contract.
    pub async fn leave(&self, contract_id: Uuid, user_id: Uuid) {
        let mut rooms = self.rooms.write().await;

        if let Some(room) = rooms.get_mut(&contract_id) {
            // Remove the first matching handle for this user.
            // (A user could have multiple connections, so only remove one.)
            if let Some(pos) = room.iter().position(|c| c.user_id == user_id) {
                room.remove(pos);
            }

            // Check if this user still has other connections in this room.
            let still_connected = room.iter().any(|c| c.user_id == user_id);

            if !still_connected {
                // Notify remaining participants that this user went offline.
                let presence_msg = ServerMessage::Presence {
                    user_id,
                    online: false,
                };
                for client in room.iter() {
                    let _ = client.sender.send(presence_msg.clone());
                }
            }

            // Clean up empty rooms.
            if room.is_empty() {
                rooms.remove(&contract_id);
            }
        }
    }

    /// Broadcast a message to all participants in a contract chat, optionally
    /// excluding the sender.
    pub async fn broadcast(
        &self,
        contract_id: Uuid,
        message: ServerMessage,
        exclude_user: Option<Uuid>,
    ) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(&contract_id) {
            for client in room {
                if Some(client.user_id) == exclude_user {
                    continue;
                }
                // If the send fails, the receiver has been dropped (disconnected).
                // That's okay — the leave() method will clean it up.
                let _ = client.sender.send(message.clone());
            }
        }
    }

    /// Send a message to all connections of a specific user in a contract.
    pub async fn send_to_user(
        &self,
        contract_id: Uuid,
        user_id: Uuid,
        message: ServerMessage,
    ) {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(&contract_id) {
            for client in room {
                if client.user_id == user_id {
                    let _ = client.sender.send(message.clone());
                }
            }
        }
    }

    /// Check if a specific user is currently online in a contract chat.
    pub async fn is_user_online(&self, contract_id: Uuid, user_id: Uuid) -> bool {
        let rooms = self.rooms.read().await;
        rooms
            .get(&contract_id)
            .map(|room| room.iter().any(|c| c.user_id == user_id))
            .unwrap_or(false)
    }
}
