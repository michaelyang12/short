use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
mod handler;
mod config;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let configuration = match config::configure_app() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {e}");
            std::process::exit(1);
        }
    };

    let pool = PgPool::connect(&configuration.database_url)
        .await
        .expect("Failed to connect to postgres database");

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(handler::root))
        .route("/shorten", post(handler::shorten))
        .route("/r/:key", get(handler::redirect))
        .route("/delete", post(handler::delete))
        .route("/healthz/live", get(|| async { "ok" }))
        .route("/healthz/ready", get(|| async { "ok" }))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let port = configuration.port;
    tracing::info!("Starting server…");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on port http://localhost:{port}");
    axum::serve(listener, app).await?;
    tracing::info!("Server returned");
    Ok(())
}