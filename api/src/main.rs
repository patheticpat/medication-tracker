pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod scheduler;

use crate::{
    handlers::{
        health::status,
        medications::{
            create_log_entry, create_medication, delete_medication, list_medications,
            medication_details, patch_snooze, update_medication,
        },
        push::{
            get_settings, get_vapid_public_key, subscribe, test_push, unsubscribe, update_settings,
        },
    },
    middleware::CustomClaims,
    scheduler::run,
};
use axum::{
    Router,
    routing::{get, patch, post},
};
use axum::{
    extract::FromRef,
    http::{
        HeaderValue,
        header::{AUTHORIZATION, CONTENT_TYPE, HeaderName},
    },
};
use axum_jwt_auth::{Decoder, RemoteJwksDecoder};
use color_eyre::Result;
use jsonwebtoken::{Algorithm, Validation};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{env, str::FromStr, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::{Any, CorsLayer};
use web_push::{
    IsahcWebPushClient, PartialVapidSignatureBuilder, URL_SAFE_NO_PAD, VapidSignatureBuilder,
};

const DATABASE_URL: &str = "DATABASE_URL";
const CORS_ORIGIN: &str = "CORS_ORIGIN";

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: SqlitePool,
    pub vapid_subject: String,
    pub vapid_signature_builder: PartialVapidSignatureBuilder,
    pub push_client: IsahcWebPushClient,
    pub decoder: Decoder<CustomClaims>,
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

    let cors_origin = env::var(CORS_ORIGIN)?;

    let vapid_private_key = env::var("VAPID_PRIVATE_KEY")?;
    let vapid_subject = env::var("VAPID_SUBJECT")?;

    let vapid_signature_builder =
        VapidSignatureBuilder::from_base64_no_sub(&vapid_private_key, URL_SAFE_NO_PAD).unwrap();

    // Set the validation parameters, as of jsonwebtoken version 9, you MUST set the algorithm and the audience
    let audience = env::var("AUTH0_AUDIENCE")?;
    let domain = env::var("AUTH0_DOMAIN")?;
    let issuer = format!("https://{}/", &domain);
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&audience]);
    validation.set_issuer(&[&issuer]);

    let jwks_url = format!("https://{}/.well-known/jwks.json", &domain);

    // Create a decoder pointing to the JWKS endpoint
    let decoder = RemoteJwksDecoder::builder()
        .jwks_url(jwks_url)
        .validation(validation)
        .build()
        .expect("Failed to build JWKS decoder");
    let decoder = Arc::new(decoder);
    decoder.initialize().await?;

    let state = AppState {
        pool,
        vapid_signature_builder,
        vapid_subject,
        push_client: IsahcWebPushClient::new()?,
        decoder,
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
        .route("/health", get(status))
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
        .route("/push/settings", get(get_settings).put(update_settings))
        .route("/push/test", post(test_push))
        .with_state(state);

    let app = app.layer(
        CorsLayer::new()
            .allow_origin(cors_origin.parse::<HeaderValue>().unwrap())
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
