pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod models;

use crate::handlers::medications::{
    create_log_entry, create_medication, delete_medication, health, list_medications,
    medication_details, update_medication,
};
use axum::{
    Router,
    routing::{get, post},
};
use color_eyre::Result;
use sqlx::SqlitePool;
use std::env;

const DATABASE_URL: &str = "DATABASE_URL";

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    let url = env::var(DATABASE_URL)?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(health))
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
