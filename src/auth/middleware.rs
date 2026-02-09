use actix_web::FromRequest;
use actix_web::{Error, HttpRequest, dev::Payload, web};
use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::auth::jwks::JwksCache;
use crate::auth::jwt;
use crate::db::users::find_or_create_from_auth;
use crate::models::users::{self, CreateUserFromAuth, Roles};

pub struct AuthenticatedUser(pub users::Model);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            // 1. Extract the Bearer token from the Authorization header.
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized("Missing Authorization header")
                })?;

            let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("Authorization header must be: Bearer <token>")
            })?;

            // 2. Get JWKS cache from app data
            let jwks_cache = req.app_data::<web::Data<Arc<JwksCache>>>().ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("JWKS cache not configured")
            })?;

            // 3. Validate the JWT using JWKS
            let claims = jwt::validate_token(token, jwks_cache.get_ref())
                .await
                .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Invalid token: {e}")))?;

            // 4. Extract user info from claims.
            let user_id = claims
                .user_id()
                .map_err(actix_web::error::ErrorUnauthorized)?;

            let email = claims
                .user_email()
                .ok_or_else(|| actix_web::error::ErrorUnauthorized("No email in token claims"))?;

            // 5. Get the database connection.
            let db = req
                .app_data::<web::Data<DatabaseConnection>>()
                .ok_or_else(|| {
                    actix_web::error::ErrorInternalServerError("Database not configured")
                })?;

            // 6. Find or create the user.
            let user = find_or_create_from_auth(
                db.get_ref(),
                CreateUserFromAuth {
                    id: user_id,
                    email,
                    display_name: claims.display_name(),
                    avatar_url: claims.avatar_url(),
                    auth_provider: "google".to_string(),
                    role: Roles::Client, // default role for new users
                },
            )
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Database error: {e}"))
            })?;

            Ok(AuthenticatedUser(user))
        })
    }
}

/// Wrapper type to store the JWT secret in Actix app data.
#[derive(Clone)]
pub struct JwtSecret(pub String);
