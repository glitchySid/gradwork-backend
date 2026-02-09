use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::users;
use crate::models::users::{CompleteProfile, UserResponse};

/// GET /api/auth/me — return the currently authenticated user's profile.
pub async fn me(user: AuthenticatedUser) -> impl Responder {
    HttpResponse::Ok().json(UserResponse::from(user.0))
}

/// POST /api/auth/complete-profile — set username, role, display_name after first login.
pub async fn complete_profile(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CompleteProfile>,
) -> impl Responder {
    match users::complete_profile(db.get_ref(), user.0.id, body.into_inner()).await {
        Ok(updated) => HttpResponse::Ok().json(UserResponse::from(updated)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update profile: {e}"),
        })),
    }
}
