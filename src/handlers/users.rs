use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::users as user_db;
use crate::models::users::{UpdateUser, UserResponse};

/// GET /api/users — list all users (requires authentication).
pub async fn get_users(
    _user: AuthenticatedUser, // ensures caller is authenticated
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    match user_db::get_all_users(db.get_ref()).await {
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
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
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

/// PUT /api/users/{id} — update a user (requires authentication).
pub async fn update_user(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
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
        Ok(updated) => HttpResponse::Ok().json(UserResponse::from(updated)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update user: {e}"),
        })),
    }
}

/// DELETE /api/users/{id} — delete a user (requires authentication).
pub async fn delete_user(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
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
