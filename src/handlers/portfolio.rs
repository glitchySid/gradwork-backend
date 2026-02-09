use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::portfolio as portfolio_db;
use crate::models::portfolio::{CreatePortfolio, UpdatePortfolio};

/// GET /api/portfolios — list all portfolio items (requires authentication).
pub async fn get_portfolios(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    match portfolio_db::get_all_portfolios(db.get_ref()).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch portfolios: {e}"),
        })),
    }
}

/// GET /api/portfolios/{id} — get a single portfolio item (requires authentication).
pub async fn get_portfolio(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    match portfolio_db::get_portfolio_by_id(db.get_ref(), id).await {
        Ok(Some(item)) => HttpResponse::Ok().json(item),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Portfolio item {id} not found"),
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

/// GET /api/portfolios/freelancer/{freelancer_id} — list portfolio items for a freelancer.
pub async fn get_portfolios_by_freelancer(
    _user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let freelancer_id = path.into_inner();
    match portfolio_db::get_portfolios_by_freelancer(db.get_ref(), freelancer_id).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to fetch portfolios: {e}"),
        })),
    }
}

/// POST /api/portfolios — create a new portfolio item (requires authentication).
pub async fn create_portfolio(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreatePortfolio>,
) -> impl Responder {
    let input = body.into_inner();

    // Only allow users to create portfolio items for themselves.
    if auth_user.0.id != input.freelancer_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only create portfolio items for your own account",
        }));
    }

    match portfolio_db::insert_portfolio(db.get_ref(), input).await {
        Ok(item) => HttpResponse::Created().json(item),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create portfolio item: {e}"),
        })),
    }
}

/// PUT /api/portfolios/{id} — update a portfolio item (requires authentication).
pub async fn update_portfolio(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
    body: web::Json<UpdatePortfolio>,
) -> impl Responder {
    let id = path.into_inner();

    // Verify the portfolio item belongs to the authenticated user.
    match portfolio_db::get_portfolio_by_id(db.get_ref(), id).await {
        Ok(Some(item)) if item.freelancer_id != auth_user.0.id => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "You can only update your own portfolio items",
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Portfolio item {id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
        _ => {}
    }

    match portfolio_db::update_portfolio(db.get_ref(), id, body.into_inner()).await {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update portfolio item: {e}"),
        })),
    }
}

/// DELETE /api/portfolios/{id} — delete a portfolio item (requires authentication).
pub async fn delete_portfolio(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();

    // Verify the portfolio item belongs to the authenticated user.
    match portfolio_db::get_portfolio_by_id(db.get_ref(), id).await {
        Ok(Some(item)) if item.freelancer_id != auth_user.0.id => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "You can only delete your own portfolio items",
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Portfolio item {id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
        _ => {}
    }

    match portfolio_db::delete_portfolio(db.get_ref(), id).await {
        Ok(result) => {
            if result.rows_affected > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Portfolio item {id} deleted"),
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Portfolio item {id} not found"),
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete portfolio item: {e}"),
        })),
    }
}
