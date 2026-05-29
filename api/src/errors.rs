use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    NotFound,
    InternalError,
    Unauthorized,
    BadRequest(String),
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            AppError::Database(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response()
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg).into_response(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Database(value)
    }
}

impl From<color_eyre::Report> for AppError {
    fn from(_: color_eyre::Report) -> Self {
        AppError::InternalError
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {e}"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::InternalError => write!(f, "Internal error"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            AppError::Conflict(msg) => write!(f, "Conflict: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}
