pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod utils;

use crate::handlers::{
    auth::{change_password, login, register},
    medications::{
        create_log_entry, create_medication, delete_medication, health, list_medications,
        medication_details, patch_snooze, update_medication,
    },
    passkey::{
        delete_passkey, list_passkeys, login_begin, login_complete, register_begin,
        register_complete,
    },
    push::{get_vapid_public_key, subscribe, unsubscribe, update_settings},
};
#[cfg(debug_assertions)]
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, HeaderName};
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use color_eyre::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{env, str::FromStr, sync::Arc};
#[cfg(debug_assertions)]
use tower_http::cors::{Any, CorsLayer};
use webauthn_rs::{Webauthn, WebauthnBuilder, prelude::Url};

const DATABASE_URL: &str = "DATABASE_URL";
const JWT_SECRET: &str = "JWT_SECRET";
const RP_ID: &str = "RP_ID";
const RP_ORIGIN: &str = "RP_ORIGIN";

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
    pub webauthn: Arc<Webauthn>,
    pub vapid_public_key: String,
    pub vapid_private_key: String,
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

    let rp_id = env::var(RP_ID).unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = env::var(RP_ORIGIN).unwrap_or_else(|_| "http://localhost:5173".to_string());
    let rp_origin = Url::parse(&rp_origin)?;
    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)?
        .rp_name("Medication Tracker")
        .build()?;
    let vapid_public_key = env::var("VAPID_PUBLIC_KEY")?;
    let vapid_private_key = env::var("VAPID_PRIVATE_KEY")?;

    let state = AppState {
        pool,
        jwt_secret,
        webauthn: Arc::new(webauthn),
        vapid_public_key,
        vapid_private_key,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/password", put(change_password))
        .route("/auth/passkey/register/begin", post(register_begin))
        .route("/auth/passkey/register/complete", post(register_complete))
        .route("/auth/passkey/login/begin", post(login_begin))
        .route("/auth/passkey/login/complete", post(login_complete))
        .route("/auth/passkeys", get(list_passkeys))
        .route("/auth/passkeys/{credential_id}", delete(delete_passkey))
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
        .route("/medications/{id}/snooze", patch(patch_snooze))
        .route("/medications/{id}/log", post(create_log_entry))
        .route("/push/vapid-public-key", get(get_vapid_public_key))
        .route("/push/subscribe", post(subscribe).delete(unsubscribe))
        .route("/push/settings", put(update_settings))
        .with_state(state);

    #[cfg(debug_assertions)]
    let app = app.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers([
                AUTHORIZATION,
                CONTENT_TYPE,
                HeaderName::from_static("x-timezone"),
            ]),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
