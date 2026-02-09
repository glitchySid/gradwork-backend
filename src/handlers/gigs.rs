use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::gigs as gig_db;
use crate::models::gigs::{CreateGig, UpdateGig};

/// GET /api/gigs — list all gigs (requires authentication).
pub async fn get_gigs(
    _user: AuthenticatedUser,
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
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
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

/// POST /api/gigs — create a new gig (requires authentication).
pub async fn create_gig(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateGig>,
) -> impl Responder {
    let user_id = user.0.id; // really don't know is this will work
    match gig_db::insert_gig(db.get_ref(), body.into_inner(), user_id).await {
        Ok(gig) => HttpResponse::Created().json(gig),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create gig: {e}"),
        })),
    }
}

/// PUT /api/gigs/{id} — update a gig (requires authentication).
pub async fn update_gig(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateGig>,
) -> impl Responder {
    let id = path.into_inner();
    match gig_db::update_gig(db.get_ref(), id, body.into_inner()).await {
        Ok(updated) => HttpResponse::Ok().json(updated),
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
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match gig_db::delete_gig(db.get_ref(), id).await {
        Ok(result) => {
            if result.rows_affected > 0 {
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
