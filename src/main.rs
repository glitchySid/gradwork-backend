use actix_cors::Cors;
// use actix_files::Files;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use gradwork_backend::auth::jwks::JwksCache;
use gradwork_backend::cache::RedisCache;
use gradwork_backend::chat::server::ChatServer;
use gradwork_backend::create_pool;
use gradwork_backend::handlers;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let db = create_pool().await;
    let db_data = web::Data::new(db);

    // Initialize Redis cache
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_cache = RedisCache::new(&redis_url)
        .await
        .expect("Failed to connect to Redis");
    let redis_data = web::Data::new(Arc::new(redis_cache));
    tracing::info!("Connected to Redis");

    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let project_ref = supabase_url
        .strip_prefix("https://")
        .and_then(|s| s.strip_suffix(".supabase.co"))
        .expect("Invalid SUPABASE_URL format. Expected: https://PROJECT.supabase.co");

    let supabase_anon_key =
        std::env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY must be set");
    let jwks_cache = web::Data::new(Arc::new(JwksCache::new(project_ref, &supabase_anon_key)));

    // Create the shared chat server (room manager for WebSocket connections).
    let chat_server = web::Data::new(Arc::new(ChatServer::new()));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("0.0.0.0:{port}");
    tracing::info!("Server running at http://{bind_addr}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .app_data(redis_data.clone())
            .app_data(jwks_cache.clone())
            .app_data(chat_server.clone())
            .service(web::scope("/api").configure(handlers::init_routes))
        // .service(Files::new("/", "./frontend").index_file("index.html"))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
