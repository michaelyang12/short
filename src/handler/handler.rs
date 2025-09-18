use crate::AppState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{Json, extract::State, http};
use axum::extract::Path;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Deserialize)]
pub struct ShortenReq {
    url: String,
    length: Option<usize>,
}

#[derive(Deserialize)]
struct RedirectReq {
    key: String,
}

#[derive(Serialize)]
struct ShortenRes {
    short: String,
    base_url: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorRes {
    message: String,
}

pub async fn root() -> impl IntoResponse {
    Json(serde_json::json!({ "message": "Rust URL Shortener API is running" }))
}

pub async fn shorten(
    State(state): State<AppState>,
    Json(body): Json<ShortenReq>,
) -> impl IntoResponse {
    let url = body.url;
    if !is_valid_url(&url) {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "message": "Invalid URL"
        })));
    }
    let length = body.length.unwrap_or(10);
    let key = nanoid!(length);

    match sqlx::query!(
        r#"INSERT INTO urls ("key", url) VALUES ($1, $2) RETURNING "key""#,
        key,
        url
    )
        .fetch_one(&state.pool)
        .await
    {
        Ok(record) => (StatusCode::CREATED, Json(json! ({
            "short": format!("/r/{}", record.key),
            "base_url": url,
            "message": format!("Successfully shortened. Access at /r/{}", record.key),
        }))),
        Err(e) => {
            tracing::error!(?e, "Failed to insert URL");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": "Internal server error"})))
        }
    }
}

pub async fn redirect(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match sqlx::query!(r#"SELECT url FROM urls WHERE key = $1"#, key)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(record)) => {
            tracing::info!("Redirecting to {}", &record.url);
            Redirect::temporary(&record.url).into_response()
        },
        Ok(None) => {
            tracing::warn!("Requested key {key} not found");
            (StatusCode::NOT_FOUND, "Short URL not found").into_response()
        },
        Err(e) => {
            tracing::error!(?e, "Database error");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn is_valid_url(s: &str) -> bool {
    match Url::parse(s) {
        Ok(u) => matches!(u.scheme(), "http" | "https") && u.has_host(),
        Err(_) => false,
    }
}
