use redis::{aio::ConnectionManager, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RedisCache {
    connection: ConnectionManager,
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;
        Ok(Self { connection })
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> redis::RedisResult<Option<T>> {
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut self.connection.clone())
            .await?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Deserialization error",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with optional TTL (in seconds)
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: Option<u64>,
    ) -> redis::RedisResult<()> {
        let serialized = serde_json::to_string(value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization error",
                e.to_string(),
            ))
        })?;

        let mut cmd = redis::cmd("SET");
        cmd.arg(key).arg(serialized);

        if let Some(ttl) = ttl_seconds {
            cmd.arg("EX").arg(ttl);
        }

        cmd.query_async(&mut self.connection.clone()).await
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> redis::RedisResult<()> {
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut self.connection.clone())
            .await
    }

    /// Delete multiple keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> redis::RedisResult<()> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut self.connection.clone())
            .await?;

        if !keys.is_empty() {
            let _: () = redis::cmd("DEL")
                .arg(&keys)
                .query_async(&mut self.connection.clone())
                .await?;
        }

        Ok(())
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> redis::RedisResult<bool> {
        redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut self.connection.clone())
            .await
    }

    /// Get remaining TTL for a key in seconds
    pub async fn ttl(&self, key: &str) -> redis::RedisResult<i64> {
        redis::cmd("TTL")
            .arg(key)
            .query_async(&mut self.connection.clone())
            .await
    }
}

/// Cache key generators
pub mod keys {
    /// Generate key for gig listings
    pub fn gig_list(filters: &str) -> String {
        format!("gigs:list:{}", filters)
    }

    /// Generate key for single gig
    pub fn gig(id: &str) -> String {
        format!("gig:{}", id)
    }

    /// Generate key for user profile
    pub fn user(id: &str) -> String {
        format!("user:{}", id)
    }

    /// Generate key for user gigs
    pub fn user_gigs(user_id: &str) -> String {
        format!("user:{}:gigs", user_id)
    }

    /// Generate key for portfolio items
    pub fn portfolio(user_id: &str) -> String {
        format!("portfolio:{}", user_id)
    }

    /// Generate key for chat conversations
    pub fn conversations(user_id: &str) -> String {
        format!("conversations:{}", user_id)
    }

    /// Generate key for messages in a conversation
    pub fn messages(conversation_id: &str) -> String {
        format!("messages:{}", conversation_id)
    }
}

/// Cache configuration
pub struct CacheConfig {
    pub gig_list_ttl: Duration,
    pub gig_ttl: Duration,
    pub user_ttl: Duration,
    pub conversation_ttl: Duration,
    pub message_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            gig_list_ttl: Duration::from_secs(300),      // 5 minutes
            gig_ttl: Duration::from_secs(600),           // 10 minutes
            user_ttl: Duration::from_secs(900),          // 15 minutes
            conversation_ttl: Duration::from_secs(300),  // 5 minutes
            message_ttl: Duration::from_secs(60),        // 1 minute
        }
    }
}

impl CacheConfig {
    pub fn from_env() -> Self {
        Self {
            gig_list_ttl: parse_duration_secs("CACHE_TTL_GIGS", 300),
            gig_ttl: parse_duration_secs("CACHE_TTL_GIG_DETAIL", 600),
            user_ttl: parse_duration_secs("CACHE_TTL_USERS", 900),
            conversation_ttl: parse_duration_secs("CACHE_TTL_CONVERSATIONS", 300),
            message_ttl: parse_duration_secs("CACHE_TTL_MESSAGES", 60),
        }
    }
}

fn parse_duration_secs(env_var: &str, default: u64) -> Duration {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(default))
}

/// Wrapper type for Actix-web app data
pub type CacheData = Arc<RedisCache>;
