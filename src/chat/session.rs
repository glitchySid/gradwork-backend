use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::Message;
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::auth::jwks::JwksCache;
use crate::auth::jwt;
use crate::chat::protocol::{ClientMessage, ServerMessage};
use crate::chat::server::ChatServer;
use crate::db::contracts as contract_db;
use crate::db::gigs as gig_db;
use crate::db::messages as message_db;
use crate::models::contracts::Status;
use crate::models::messages::CreateMessage;

/// Query params for the WebSocket handshake endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// GET /api/chat/ws/{contract_id}?token=<jwt>
///
/// Upgrades the HTTP connection to a WebSocket.
/// Authenticates via query param token (browsers can't send Authorization headers
/// during the WebSocket handshake).
/// Validates that:
/// 1. The JWT is valid.
/// 2. The contract exists and is Accepted.
/// 3. The user is a party to the contract (client or gig owner/freelancer).
pub async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<Uuid>,
    query: web::Query<WsQuery>,
    db: web::Data<DatabaseConnection>,
    jwks_cache: web::Data<Arc<JwksCache>>,
    chat_server: web::Data<Arc<ChatServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    let contract_id = path.into_inner();
    let token = &query.token;

    // 1. Validate the JWT.
    let claims = jwt::validate_token(token, jwks_cache.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Invalid token: {e}")))?;

    let user_id = claims
        .user_id()
        .map_err(actix_web::error::ErrorUnauthorized)?;

    // 2. Fetch the contract and verify it's Accepted.
    let contract = contract_db::get_contract_by_id(db.get_ref(), contract_id)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Database error: {e}"))
        })?
        .ok_or_else(|| {
            actix_web::error::ErrorNotFound(format!("Contract {contract_id} not found"))
        })?;

    if contract.status != Status::Accepted {
        return Err(actix_web::error::ErrorForbidden(
            "Chat is only available for accepted contracts",
        ));
    }

    // 3. Verify the user is a party to the contract.
    let is_client = contract.user_id == user_id;
    let is_freelancer = match gig_db::get_gig_by_id(db.get_ref(), contract.gig_id).await {
        Ok(Some(gig)) => gig.user_id == user_id,
        _ => false,
    };

    if !is_client && !is_freelancer {
        return Err(actix_web::error::ErrorForbidden(
            "You are not a party to this contract",
        ));
    }

    // 4. Upgrade to WebSocket.
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // 5. Join the chat room and get a receiver for outgoing messages.
    let rx = chat_server.join(contract_id, user_id).await;

    // 6. Spawn the WebSocket session task.
    let db_clone = db.get_ref().clone();
    let chat_server_clone = chat_server.get_ref().clone();

    actix_web::rt::spawn(handle_ws_session(
        session,
        msg_stream,
        rx,
        contract_id,
        user_id,
        db_clone,
        chat_server_clone,
    ));

    Ok(response)
}

/// Drives the WebSocket session: reads incoming messages from the client,
/// sends outgoing messages from the chat server, and handles cleanup on disconnect.
async fn handle_ws_session(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    mut rx: mpsc::UnboundedReceiver<ServerMessage>,
    contract_id: Uuid,
    user_id: Uuid,
    db: DatabaseConnection,
    chat_server: Arc<ChatServer>,
) {
    loop {
        tokio::select! {
            // Incoming message from the WebSocket client.
            Some(msg) = msg_stream.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        handle_client_message(
                            &text,
                            &mut session,
                            contract_id,
                            user_id,
                            &db,
                            &chat_server,
                        )
                        .await;
                    }
                    Ok(Message::Ping(bytes)) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            // Outgoing message from the chat server to this client.
            Some(server_msg) = rx.recv() => {
                let json = match serde_json::to_string(&server_msg) {
                    Ok(j) => j,
                    Err(_) => continue,
                };
                if session.text(json).await.is_err() {
                    break;
                }
            }
            // Both channels closed — exit.
            else => break,
        }
    }

    // Clean up: leave the chat room.
    chat_server.leave(contract_id, user_id).await;
    let _ = session.close(None).await;
}

/// Parse and handle an incoming client message.
async fn handle_client_message(
    text: &str,
    session: &mut actix_ws::Session,
    contract_id: Uuid,
    user_id: Uuid,
    db: &DatabaseConnection,
    chat_server: &ChatServer,
) {
    let client_msg: ClientMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            let err = ServerMessage::Error {
                message: format!("Invalid message format: {e}"),
            };
            let _ = session
                .text(serde_json::to_string(&err).unwrap_or_default())
                .await;
            return;
        }
    };

    match client_msg {
        ClientMessage::SendMessage { content } => {
            if content.trim().is_empty() {
                let err = ServerMessage::Error {
                    message: "Message content cannot be empty".to_string(),
                };
                let _ = session
                    .text(serde_json::to_string(&err).unwrap_or_default())
                    .await;
                return;
            }

            // Persist the message to the database.
            let input = CreateMessage {
                contract_id,
                sender_id: user_id,
                content: content.clone(),
            };

            match message_db::insert_message(db, input).await {
                Ok(saved) => {
                    let msg = ServerMessage::NewMessage {
                        id: saved.id,
                        sender_id: saved.sender_id,
                        content: saved.content,
                        created_at: saved.created_at.to_rfc3339(),
                    };

                    // Broadcast to all participants (including sender, so they
                    // get the server-assigned id and timestamp).
                    chat_server.broadcast(contract_id, msg, None).await;
                }
                Err(e) => {
                    let err = ServerMessage::Error {
                        message: format!("Failed to save message: {e}"),
                    };
                    let _ = session
                        .text(serde_json::to_string(&err).unwrap_or_default())
                        .await;
                }
            }
        }

        ClientMessage::MarkRead { message_id } => {
            match message_db::mark_message_as_read(db, message_id).await {
                Ok(_) => {
                    // Notify all participants that this message was read.
                    let msg = ServerMessage::MessageRead { message_id };
                    chat_server.broadcast(contract_id, msg, None).await;
                }
                Err(e) => {
                    let err = ServerMessage::Error {
                        message: format!("Failed to mark message as read: {e}"),
                    };
                    let _ = session
                        .text(serde_json::to_string(&err).unwrap_or_default())
                        .await;
                }
            }
        }

        ClientMessage::Typing => {
            let msg = ServerMessage::UserTyping { user_id };
            // Only send to others — the sender already knows they're typing.
            chat_server
                .broadcast(contract_id, msg, Some(user_id))
                .await;
        }

        ClientMessage::StopTyping => {
            let msg = ServerMessage::UserStopTyping { user_id };
            chat_server
                .broadcast(contract_id, msg, Some(user_id))
                .await;
        }
    }
}
