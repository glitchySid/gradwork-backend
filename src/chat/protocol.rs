use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Client -> Server messages ──

/// Messages the client sends to the server over WebSocket.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message.
    SendMessage { content: String },
    /// Mark a specific message as read.
    MarkRead { message_id: Uuid },
    /// Notify the other party that the user is typing.
    Typing,
    /// Notify the other party that the user stopped typing.
    StopTyping,
}

// ── Server -> Client messages ──

/// Messages the server sends to the client over WebSocket.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// A new message was received (or echo of the sender's own message).
    NewMessage {
        id: Uuid,
        sender_id: Uuid,
        content: String,
        created_at: String,
    },
    /// A message was marked as read.
    MessageRead { message_id: Uuid },
    /// The other user is typing.
    UserTyping { user_id: Uuid },
    /// The other user stopped typing.
    UserStopTyping { user_id: Uuid },
    /// Presence update: a user came online or went offline in this contract chat.
    Presence { user_id: Uuid, online: bool },
    /// An error occurred.
    Error { message: String },
}
