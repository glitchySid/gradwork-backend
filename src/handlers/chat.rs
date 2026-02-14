use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;
use tracing;

use crate::auth::authorization::verify_contract_party;
use crate::auth::middleware::AuthenticatedUser;
use crate::cache::{RedisCache, keys};
use crate::db::contracts as contract_db;
use crate::db::gigs as gig_db;
use crate::db::messages as message_db;
use crate::models::contracts::Status;
use crate::models::messages::{ConversationSummary, MessageQuery, MessageResponse};

/// GET /api/chat/{contract_id}/messages?page=1&limit=50
///
/// Fetch paginated message history for a contract.
/// Only the two parties of the contract can access this.
pub async fn get_messages(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
    query: web::Query<MessageQuery>,
) -> impl Responder {
    let contract_id = path.into_inner();
    let user_id = user.0.id;

    if let Err(resp) = verify_contract_party(db.get_ref(), contract_id, user_id).await {
        return resp;
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let cache_key = format!("messages:{contract_id}:{page}:{limit}");

    match cache.get::<Vec<MessageResponse>>(&cache_key).await {
        Ok(Some(cached)) => return HttpResponse::Ok().json(cached),
        Ok(None) => {}
        Err(e) => tracing::warn!("Cache error: {}", e),
    }

    match message_db::get_messages_by_contract(db.get_ref(), contract_id, page, limit).await {
        Ok(messages) => {
            let response: Vec<MessageResponse> = messages.into_iter().map(|m| m.into()).collect();
            let _ = cache.set(&cache_key, &response, Some(60)).await;
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

/// PUT /api/chat/messages/{id}/read
///
/// Mark a specific message as read. Only the recipient (non-sender) should call this.
pub async fn mark_message_read(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let message_id = path.into_inner();
    let user_id = user.0.id;

    match message_db::mark_message_as_read(db.get_ref(), message_id).await {
        Ok(msg) => {
            let _ = cache
                .delete(&keys::conversations(&user_id.to_string()))
                .await;
            let response: MessageResponse = msg.into();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to mark message as read: {e}"),
        })),
    }
}

/// GET /api/chat/conversations
///
/// List all contracts with chat activity for the authenticated user.
/// Returns a summary with the last message, unread count, and the other party's info.
pub async fn get_conversations(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
) -> impl Responder {
    let user_id = user.0.id;
    let cache_key = keys::conversations(&user_id.to_string());

    match cache.get::<Vec<ConversationSummary>>(&cache_key).await {
        Ok(Some(cached)) => return HttpResponse::Ok().json(cached),
        Ok(None) => {}
        Err(e) => tracing::warn!("Cache error: {}", e),
    }

    // Get all contracts where the user is the client.
    let as_client = match contract_db::get_contracts_by_user_id(db.get_ref(), user_id).await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // Get all contracts where the user is the freelancer (gig owner).
    let user_gigs = match gig_db::get_gigs_by_user_id(db.get_ref(), user_id).await {
        Ok(g) => g,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // Batch fetch all contracts for user's gigs in a single query (N+1 fix)
    let gig_ids: Vec<Uuid> = user_gigs.iter().map(|g| g.id).collect();
    let freelancer_contracts = match contract_db::get_contracts_by_gig_ids(db.get_ref(), gig_ids).await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // Merge and deduplicate, keeping only Accepted contracts.
    let mut all_contracts = as_client;
    for contract in freelancer_contracts {
        if !all_contracts.iter().any(|c| c.id == contract.id) {
            all_contracts.push(contract);
        }
    }
    let accepted_contracts: Vec<_> = all_contracts
        .into_iter()
        .filter(|c| c.status == Status::Accepted)
        .collect();

    // Build conversation summaries.
    let mut summaries: Vec<ConversationSummary> = Vec::new();

    for contract in accepted_contracts {
        // Determine the other user's ID.
        let other_user_id = if contract.user_id == user_id {
            // Current user is the client — the other party is the freelancer (gig owner).
            match gig_db::get_gig_by_id(db.get_ref(), contract.gig_id).await {
                Ok(Some(gig)) => gig.user_id,
                _ => continue,
            }
        } else {
            // Current user is the freelancer — the other party is the client.
            contract.user_id
        };

        // Get the other user's display name.
        let other_user_name =
            match crate::db::users::get_user_by_id(db.get_ref(), other_user_id).await {
                Ok(Some(u)) => u.display_name,
                _ => None,
            };

        // Get the latest message and unread count.
        let latest_msg =
            message_db::get_latest_message_for_contract(db.get_ref(), contract.id).await;
        let unread = message_db::count_unread_for_contract(db.get_ref(), contract.id, user_id)
            .await
            .unwrap_or(0);

        let (last_message, last_message_at) = match latest_msg {
            Ok(Some(msg)) => (Some(msg.content), Some(msg.created_at)),
            _ => (None, None),
        };

        summaries.push(ConversationSummary {
            contract_id: contract.id,
            other_user_id,
            other_user_name,
            last_message,
            last_message_at,
            unread_count: unread,
        });
    }

    summaries.sort_by(|a, b| {
        let a_time = a.last_message_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
        let b_time = b.last_message_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });

    let _ = cache.set(&cache_key, &summaries, Some(300)).await;
    HttpResponse::Ok().json(summaries)
}
