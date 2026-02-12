use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::cache::{keys, RedisCache};
use crate::db::gigs as gig_db;
use crate::models::gigs::{CreateGig, UpdateGig};

/// GET /api/gigs — list all gigs (requires authentication).
pub async fn get_gigs(
    // _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    match gig_db::get_all_gigs(db.get_ref()).await {
        Ok(gigs) => HttpResponse::Ok().json(gigs),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch gigs: {e}"),
        })),
    }
}

/// GET /api/gigs/{id} — get a single gig (requires authentication).
pub async fn get_gig(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    let cache_key = keys::gig(&id.to_string());

    // Try to get from cache first
    match cache.get::<serde_json::Value>(&cache_key).await {
        Ok(Some(cached)) => {
            return HttpResponse::Ok().json(cached);
        }
        Ok(None) => {
            // Cache miss - fetch from database
            match gig_db::get_gig_by_id(db.get_ref(), id).await {
                Ok(Some(gig)) => {
                    // Store in cache (10 minute TTL)
                    let _ = cache
                        .set(&cache_key, &gig, Some(600))
                        .await;
                    HttpResponse::Ok().json(gig)
                }
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Gig {id} not found"),
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                })),
            }
        }
        Err(e) => {
            // Cache error - fallback to database
            eprintln!("Cache error: {e}");
            match gig_db::get_gig_by_id(db.get_ref(), id).await {
                Ok(Some(gig)) => HttpResponse::Ok().json(gig),
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Gig {id} not found"),
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                })),
            }
        }
    }
}

/// GET /api/gigs/user/{user_id} — get gigs by user_id (requires authentication).
pub async fn get_gigs_by_user_id(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();
    match gig_db::get_gigs_by_user_id(db.get_ref(), user_id).await {
        Ok(gigs) => HttpResponse::Ok().json(gigs),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

/// DELETE /api/gigs/user/{user_id} — delete all gigs by user_id (requires authentication).
pub async fn delete_all_gig_by_user_id(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();
    match gig_db::delete_all_gig_by_user_id(db.get_ref(), user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

/// POST /api/gigs — create a new gig (requires authentication).
pub async fn create_gig(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    body: web::Json<CreateGig>,
) -> impl Responder {
    let user_id = user.0.id;
    match gig_db::insert_gig(db.get_ref(), body.into_inner(), user_id).await {
        Ok(gig) => {
            // Invalidate user's gigs cache and all gigs list
            let _ = cache.delete(&keys::user_gigs(&user_id.to_string())).await;
            let _ = cache.delete_pattern("gigs:list:*").await;
            HttpResponse::Created().json(gig)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create gig: {e}"),
        })),
    }
}

/// PUT /api/gigs/{id} — update a gig (requires authentication).
pub async fn update_gig(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateGig>,
) -> impl Responder {
    let id = path.into_inner();
    match gig_db::update_gig(db.get_ref(), id, body.into_inner()).await {
        Ok(updated) => {
            // Invalidate specific gig cache and related caches
            let _ = cache.delete(&keys::gig(&id.to_string())).await;
            let _ = cache.delete_pattern("gigs:list:*").await;
            HttpResponse::Ok().json(updated)
        }
        Err(e) => {
            let mut status = if e.to_string().contains("not found") {
                HttpResponse::NotFound()
            } else {
                HttpResponse::InternalServerError()
            };
            status.json(serde_json::json!({
                "error": format!("Failed to update gig: {e}"),
            }))
        }
    }
}

/// DELETE /api/gigs/{id} — delete a gig (requires authentication).
pub async fn delete_gig(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    cache: web::Data<Arc<RedisCache>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match gig_db::delete_gig(db.get_ref(), id).await {
        Ok(result) => {
            if result.rows_affected > 0 {
                // Invalidate specific gig cache and related caches
                let _ = cache.delete(&keys::gig(&id.to_string())).await;
                let _ = cache.delete_pattern("gigs:list:*").await;
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Gig {id} deleted"),
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Gig {id} not found"),
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete gig: {e}"),
        })),
    }
}
