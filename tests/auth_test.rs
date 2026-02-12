///! Integration test for JWT auth validation.
///!
///! This test mints a JWT locally using the same HS256 secret that the server
///! would use, then validates it through the `validate_token` function.
///! No running server or database is needed.
///!
///! Run with: `cargo test --test auth_test`
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use uuid::Uuid;

use gradwork_backend::auth::jwt::{Claims, UserMetadata, validate_token};

/// A fake secret for testing — never use the real one in tests committed to git.
const TEST_SECRET: &str = "test-secret-at-least-256-bits-long-for-hs256-xxxxxxx";

/// Helper: mint a JWT signed with HS256 using the test secret.
fn mint_test_token(sub: &str, email: &str, full_name: &str) -> String {
    let now = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: sub.to_string(),
        exp: now + 3600, // 1 hour from now
        iat: Some(now),
        iss: Some("https://example.supabase.co/auth/v1".to_string()),
        email: Some(email.to_string()),
        role: Some("authenticated".to_string()),
        user_metadata: Some(UserMetadata {
            full_name: Some(full_name.to_string()),
            name: None,
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            picture: None,
            email: Some(email.to_string()),
            email_verified: Some(true),
        }),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .expect("Failed to encode test JWT")
}

#[test]
fn test_valid_token_decodes_correctly() {
    let user_id = Uuid::new_v4();
    let token = mint_test_token(&user_id.to_string(), "alice@example.com", "Alice Smith");

    let claims = validate_token(&token, TEST_SECRET).expect("Token should be valid");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.user_email().unwrap(), "alice@example.com");
    assert_eq!(claims.display_name().unwrap(), "Alice Smith");
    assert_eq!(
        claims.avatar_url().unwrap(),
        "https://example.com/avatar.png"
    );
    assert_eq!(claims.user_id().unwrap(), user_id);
}

#[test]
fn test_expired_token_is_rejected() {
    let now = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        exp: now - 300, // expired 5 minutes ago (well past the 60s default leeway)
        iat: Some(now - 3600),
        iss: None,
        email: Some("expired@example.com".to_string()),
        role: None,
        user_metadata: None,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .unwrap();

    let result = validate_token(&token, TEST_SECRET);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ExpiredSignature"));
}

#[test]
fn test_wrong_secret_is_rejected() {
    let token = mint_test_token(&Uuid::new_v4().to_string(), "bob@example.com", "Bob Jones");

    let result = validate_token(&token, "completely-wrong-secret-xxxxxxxxxxxxxxxxxxx");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("InvalidSignature"));
}

#[test]
fn test_garbage_token_is_rejected() {
    let result = validate_token("not.a.valid.jwt", TEST_SECRET);
    assert!(result.is_err());
}

#[test]
fn test_claims_helpers_with_missing_metadata() {
    let now = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        exp: now + 3600,
        iat: Some(now),
        iss: None,
        email: Some("bare@example.com".to_string()),
        role: None,
        user_metadata: None, // no metadata at all
    };

    // Should fall back to top-level email.
    assert_eq!(claims.user_email().unwrap(), "bare@example.com");
    // No metadata → None.
    assert!(claims.display_name().is_none());
    assert!(claims.avatar_url().is_none());
}
