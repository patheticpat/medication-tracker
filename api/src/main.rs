pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod scheduler;
pub mod utils;

#[cfg(debug_assertions)]
use crate::handlers::push::test_push;
use crate::{
    handlers::{
        auth::{change_password, login, register},
        medications::{
            create_log_entry, create_medication, delete_medication, health, list_medications,
            medication_details, patch_snooze, update_medication,
        },
        passkey::{
            delete_passkey, list_passkeys, login_begin, login_complete, register_begin,
            register_complete,
        },
        push::{get_settings, get_vapid_public_key, subscribe, unsubscribe, update_settings},
    },
    scheduler::run,
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
use tokio_cron_scheduler::{Job, JobScheduler};
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
    pub vapid_private_key: String,
    pub vapid_subject: String,
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
    let vapid_private_key = env::var("VAPID_PRIVATE_KEY")?;
    let vapid_subject = env::var("VAPID_SUBJECT")?;

    let state = AppState {
        pool,
        jwt_secret,
        webauthn: Arc::new(webauthn),
        vapid_private_key,
        vapid_subject,
    };

    let scheduler = JobScheduler::new().await?;
    let scheduler_state = state.clone();

    scheduler
        .add(Job::new_async("0 0 * * * *", move |_, _| {
            let state = scheduler_state.clone();
            Box::pin(async move {
                if let Err(e) = run(&state).await {
                    eprintln!("scheduler error: {e}");
                }
            })
        })?)
        .await?;

    scheduler.start().await?;

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
        .route("/push/settings", get(get_settings).put(update_settings));

    #[cfg(debug_assertions)]
    let app = app.route("/push/test", post(test_push));

    let app = app.with_state(state);

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
