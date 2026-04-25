pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;

use crate::handlers::{
    auth::{login, register},
    medications::{
        create_log_entry, create_medication, delete_medication, health, list_medications,
        medication_details, update_medication,
    },
};
use axum::{
    Router,
    routing::{get, post},
};
use color_eyre::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{env, str::FromStr};
#[cfg(debug_assertions)]
use tower_http::cors::{Any, CorsLayer};

const DATABASE_URL: &str = "DATABASE_URL";
const JWT_SECRET: &str = "JWT_SECRET";

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let url = env::var(DATABASE_URL)?;
    let options = SqliteConnectOptions::from_str(&url)?
        .pragma("foreign_keys", "ON")
        .create_if_missing(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let jwt_secret = env::var(JWT_SECRET)?;
    let state = AppState { pool, jwt_secret };

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route(
            "/medications",
            get(list_medications).post(create_medication),
        )
        .route(
            "/medications/{id}",
            get(medication_details)
                .delete(delete_medication)
                .patch(update_medication),
        )
        .route("/medications/{id}/log", post(create_log_entry))
        .with_state(state);
    #[cfg(debug_assertions)]
    let app = app.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
