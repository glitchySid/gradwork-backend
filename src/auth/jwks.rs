use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use moka::future::Cache;
use std::sync::Arc;
use tracing::debug;

const JWKS_URL_TEMPLATE: &str = "https://{}.supabase.co/auth/v1/.well-known/jwks.json";

#[derive(Clone)]
struct JwksKeyData {
    x: String,
    y: String,
    algorithm: Algorithm,
}

#[derive(Clone)]
pub struct JwksCache {
    cache: Arc<Cache<String, JwksKeyData>>,
    jwks_url: String,
    client: reqwest::Client,
    anon_key: String,
}

impl JwksCache {
    pub fn new(project_ref: &str, anon_key: &str) -> Self {
        let client = reqwest::Client::new();
        let cache = Arc::new(
            Cache::builder()
                .time_to_live(std::time::Duration::from_secs(3600))
                .max_capacity(10)
                .build(),
        );

        let jwks_url = JWKS_URL_TEMPLATE.replace("{}", project_ref);

        Self {
            cache,
            jwks_url,
            client,
            anon_key: anon_key.to_string(),
        }
    }

    async fn fetch_jwks(&self) -> Result<serde_json::Value, String> {
        debug!("Fetching JWKS from {}", self.jwks_url);

        let response: reqwest::Response = self
            .client
            .get(&self.jwks_url)
            .header("apikey", &self.anon_key)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {e}"))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Failed to fetch JWKS: HTTP {status}"));
        }

        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to get JWKS text: {e}"))?;

        serde_json::from_str(&text).map_err(|e| format!("Failed to parse JWKS JSON: {e}"))
    }

    async fn get_key_data(&self, kid: &str) -> Result<JwksKeyData, String> {
        if let Some(cached) = self.cache.get(kid).await {
            return Ok(cached);
        }

        let jwks = self.fetch_jwks().await?;
        let keys = jwks["keys"].as_array().ok_or("No keys in JWKS")?;

        let key_data = keys
            .iter()
            .find(|k| k["kid"].as_str() == Some(kid))
            .ok_or(format!("Key with kid={kid} not found in JWKS"))?;

        let x = key_data["x"]
            .as_str()
            .ok_or("Missing 'x' in JWK")?
            .to_string();
        let y = key_data["y"]
            .as_str()
            .ok_or("Missing 'y' in JWK")?
            .to_string();

        let alg_str = key_data["alg"].as_str().unwrap_or("ES256");
        let algorithm = match alg_str {
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            _ => Algorithm::ES256,
        };

        let key_data = JwksKeyData { x, y, algorithm };

        self.cache.insert(kid.to_string(), key_data.clone()).await;
        Ok(key_data)
    }

    pub async fn validate_token(
        &self,
        token: &str,
    ) -> Result<TokenData<super::jwt::Claims>, String> {
        let header = decode_header(token).map_err(|e| format!("Failed to decode header: {e}"))?;
        let kid = header.kid.ok_or("No 'kid' in token header")?;

        let key_data = self.get_key_data(&kid).await?;

        let decoding_key = DecodingKey::from_ec_components(&key_data.x, &key_data.y)
            .map_err(|e| format!("Failed to create decoding key: {e}"))?;

        let mut validation = Validation::new(key_data.algorithm);
        validation.validate_aud = false;

        decode::<super::jwt::Claims>(token, &decoding_key, &validation)
            .map_err(|e| format!("Token validation failed: {e}"))
    }
}
