use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;
use tracing;

use crate::auth::middleware::AuthenticatedUser;
use crate::cache::{RedisCache, keys};
use crate::db::users as user_db;
use crate::models::users::{UpdateUser, UserResponse};
use crate::models::PaginationQuery;

/// GET /api/users — list all users with pagination (requires authentication).
/// Query params: ?page=1&limit=20
pub async fn get_users(
    _user: AuthenticatedUser, // ensures caller is authenticated
    db: web::Data<DatabaseConnection>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let page = query.page();
    let limit = query.limit();

    match user_db::get_users_paginated(db.get_ref(), page, limit).await {
        Ok(users) => {
            let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch users: {e}"),
        })),
    }
}

/// GET /api/users/{id} — get a single user (requires authentication).
pub async fn get_user(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let cache_key = keys::user(&id.to_string());

    // Try to get from cache first
    match cache.get::<serde_json::Value>(&cache_key).await {
        Ok(Some(cached)) => {
            HttpResponse::Ok().json(cached)
        }
        Ok(None) => {
            // Cache miss - fetch from database
            match user_db::get_user_by_id(db.get_ref(), id).await {
                Ok(Some(user)) => {
                    let response = UserResponse::from(user);
                    // Store in cache (15 minute TTL)
                    let _ = cache.set(&cache_key, &response, Some(900)).await;
                    HttpResponse::Ok().json(response)
                }
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("User {id} not found"),
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                })),
            }
        }
        Err(e) => {
            // Cache error - fallback to database
            tracing::warn!("Cache error: {}", e);
            match user_db::get_user_by_id(db.get_ref(), id).await {
                Ok(Some(user)) => HttpResponse::Ok().json(UserResponse::from(user)),
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("User {id} not found"),
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                })),
            }
        }
    }
}

/// PUT /api/users/{id} — update a user (requires authentication).
pub async fn update_user(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUser>,
) -> impl Responder {
    let id = path.into_inner();

    // Only allow users to update themselves (or add admin check here).
    if auth_user.0.id != id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only update your own account",
        }));
    }

    match user_db::update_user(db.get_ref(), id, body.into_inner()).await {
        Ok(updated) => {
            // Invalidate user cache and related caches
            let _ = cache.delete(&keys::user(&id.to_string())).await;
            HttpResponse::Ok().json(UserResponse::from(updated))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update user: {e}"),
        })),
    }
}

/// DELETE /api/users/{id} — delete a user (requires authentication).
pub async fn delete_user(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    // Only allow users to delete themselves (or add admin check here).
    if auth_user.0.id != id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only delete your own account",
        }));
    }

    match user_db::delete_user(db.get_ref(), id).await {
        Ok(result) => {
            if result.rows_affected > 0 {
                // Invalidate user cache and user's related caches
                let _ = cache.delete(&keys::user(&id.to_string())).await;
                let _ = cache.delete(&keys::user_gigs(&id.to_string())).await;
                let _ = cache.delete(&keys::portfolio(&id.to_string())).await;
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("User {id} deleted"),
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("User {id} not found"),
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete user: {e}"),
        })),
    }
}
