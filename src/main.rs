//! Anime Scraper API Server
//!
//! Main entry point for the anime scraper REST API service.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use anime_scraper::config::Config;
use anime_scraper::db::Database;
use anime_scraper::routes::{configure_routes, ApiDoc, AppState};

/// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Database health check endpoint
async fn db_health_check(data: web::Data<AppState>) -> impl Responder {
    match data.db.health_check().await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "database": "connected",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        Err(e) => {
            error!("Database health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let bind_address = format!("{}:{}", config.host, config.port);

    info!("Connecting to database...");
    let db = Database::new(&config.database_url)
        .await
        .expect("Failed to connect to database");

    info!("Running database migrations...");
    db.run_migrations()
        .await
        .expect("Failed to run database migrations");

    info!("Database connected and migrations complete");

    let app_state = web::Data::new(AppState {
        db,
        config: config.clone(),
    });

    info!("Starting Anime Scraper API server on {}", bind_address);

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/health/db", web::get().to(db_health_check))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            .configure(configure_routes)
    })
    .bind(&bind_address)?
    .run()
    .await
}
