use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Redirect}, routing::{get, post}};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;
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
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    
    let port = configuration.port;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on port http://localhost:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}