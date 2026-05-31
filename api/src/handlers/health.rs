use axum::{Json, response::IntoResponse};

pub async fn status() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("GIT_SHA")
    }))
}
