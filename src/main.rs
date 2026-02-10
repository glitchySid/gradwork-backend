use actix_cors::Cors;
// use actix_files::Files;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use gradwork_backend::auth::jwks::JwksCache;
use gradwork_backend::chat::server::ChatServer;
use gradwork_backend::create_pool;
use gradwork_backend::handlers;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db = create_pool().await;
    let db_data = web::Data::new(db);

    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let project_ref = supabase_url
        .strip_prefix("https://")
        .and_then(|s| s.strip_suffix(".supabase.co"))
        .expect("Invalid SUPABASE_URL format. Expected: https://PROJECT.supabase.co");

    let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY must be set");
    println!("DEBUG: project_ref = {}", project_ref);
    println!("DEBUG: anon_key prefix = {}", &supabase_anon_key[..50]);
    let jwks_cache = web::Data::new(Arc::new(JwksCache::new(project_ref, &supabase_anon_key)));

    // Create the shared chat server (room manager for WebSocket connections).
    let chat_server = web::Data::new(Arc::new(ChatServer::new()));

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
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
            .app_data(jwks_cache.clone())
            .app_data(chat_server.clone())
            .service(web::scope("/api").configure(handlers::init_routes))
            // .service(Files::new("/", "./frontend").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
