use crate::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{extract::State, Json};
use nanoid::nanoid;
use serde::Deserialize;
use serde_json::json;
use url::Url;

#[derive(Deserialize)]
pub struct ShortenReq {
    url: String,
    length: Option<usize>,
}

#[derive(Deserialize)]
pub struct DeleteReq {
    key: String,
}


pub async fn root() -> impl IntoResponse {
    Json(json!({ "message": "Rust URL Shortener API is running" }))
}

pub async fn shorten(
    State(state): State<AppState>,
    Json(body): Json<ShortenReq>,
) -> impl IntoResponse {
    let url = body.url;
    if !is_valid_url(&url) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "message": "Invalid URL"
            })),
        );
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
        Ok(record) => (
            StatusCode::CREATED,
            Json(json!({
                "short": format!("/r/{}", record.key),
                "base_url": url,
                "message": format!("Successfully shortened. Access at /r/{}", record.key),
            })),
        ),
        Err(e) => {
            tracing::error!(?e, "Failed to insert URL");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": "Internal server error"})),
            )
        }
    }
}

pub async fn redirect(State(state): State<AppState>, Path(key): Path<String>) -> impl IntoResponse {
    match sqlx::query!(r#"SELECT url, "key" FROM urls WHERE "key" = $1"#, key)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(record)) => {
            tracing::info!("Redirecting to {}", &record.url);
            bump_key(state.pool, key);
            Redirect::temporary(&record.url).into_response()
        }
        Ok(None) => {
            tracing::warn!("Requested key {key} not found");
            (StatusCode::NOT_FOUND, "Short URL not found").into_response()
        }
        Err(e) => {
            tracing::error!(?e, "Database error");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn delete(State(state): State<AppState>, Json(body): Json<DeleteReq>) -> impl IntoResponse {
    let key = body.key;
    tracing::info!("Deleting key: {}", key);
    match sqlx::query!(r#"DELETE FROM urls WHERE key = $1"#, key).execute(&state.pool).await {
        Ok(result) if result.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found", "message": "Key not found" })))
        }
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({"message": format!("Key {} deleted", key)}))),
        Err(e) => {
            tracing::error!(?e, "Failed to delete URL with key: {}", key);
            (StatusCode::BAD_REQUEST, Json(json!({ "error": "bad_request", "message": format!("Failed to delete key: {}", key)})))
        }
    }
}

fn bump_key(pool: sqlx::PgPool, key: String) {
    tokio::spawn(async move {
        tracing::info!("Bumping key: {}", &key);
        if let Err(e) = sqlx::query!(r#"UPDATE urls SET hits = hits + 1 WHERE key = $1"#, key)
            .execute(&pool)
            .await
        {
            tracing::warn!(?e, "Failed to update hits for key: {}", key);
        }
    });
}

fn is_valid_url(s: &str) -> bool {
    match Url::parse(s) {
        Ok(u) => matches!(u.scheme(), "http" | "https") && u.has_host(),
        Err(_) => false,
    }
}
