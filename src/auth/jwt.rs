use crate::auth::jwks::JwksCache;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supabase JWT claims.
///
/// Supabase issues JWTs with these standard + custom fields.
/// The `sub` field is the user's UUID in `auth.users`.
/// `user_metadata` contains profile info from the OAuth provider (Google).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// The Supabase auth user UUID.
    pub sub: String,
    /// Token expiration (Unix timestamp).
    pub exp: usize,
    /// Token issued-at (Unix timestamp).
    pub iat: Option<usize>,
    /// Issuer — should match your Supabase URL + `/auth/v1`.
    pub iss: Option<String>,
    /// User's email from Supabase auth.
    pub email: Option<String>,
    /// Supabase role (e.g. "authenticated").
    pub role: Option<String>,
    /// Metadata from the OAuth provider.
    pub user_metadata: Option<UserMetadata>,
}

/// Metadata populated by the OAuth provider (Google).
#[derive(Debug, Serialize, Deserialize)]
pub struct UserMetadata {
    pub full_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

impl Claims {
    /// Extract the user UUID from the `sub` claim.
    pub fn user_id(&self) -> Result<Uuid, String> {
        Uuid::parse_str(&self.sub).map_err(|e| format!("Invalid UUID in sub claim: {e}"))
    }

    /// Best-effort display name from metadata.
    pub fn display_name(&self) -> Option<String> {
        self.user_metadata
            .as_ref()
            .and_then(|m| m.full_name.clone().or_else(|| m.name.clone()))
    }

    /// Best-effort avatar URL from metadata.
    pub fn avatar_url(&self) -> Option<String> {
        self.user_metadata
            .as_ref()
            .and_then(|m| m.avatar_url.clone().or_else(|| m.picture.clone()))
    }

    /// Best-effort email: prefer top-level, fall back to metadata.
    pub fn user_email(&self) -> Option<String> {
        self.email
            .clone()
            .or_else(|| self.user_metadata.as_ref().and_then(|m| m.email.clone()))
    }
}

/// Validate a Supabase JWT and return the decoded claims.
///
/// Supabase signs JWTs with HS256 using the project's JWT secret.
pub async fn validate_token(token: &str, jwks_cache: &JwksCache) -> Result<Claims, String> {
    jwks_cache.validate_token(token).await.map(|td| td.claims)
}
