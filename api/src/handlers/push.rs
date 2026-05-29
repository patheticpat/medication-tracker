use axum::{Json, extract::State, http::StatusCode};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use uuid::Uuid;
use web_push::{URL_SAFE_NO_PAD, VapidSignatureBuilder};

use crate::{
    AppState,
    errors::AppError,
    extractors::Timezone,
    middleware::AuthUser,
    models::{
        DeletePushRequest, NotificationSettingsRequest, NotificationSettingsResponse,
        SubscribeRequest, VapidPublicKeyResponse,
    },
};

pub async fn get_vapid_public_key(State(state): State<AppState>) -> Json<VapidPublicKeyResponse> {
    let builder =
        VapidSignatureBuilder::from_base64_no_sub(&state.vapid_private_key, URL_SAFE_NO_PAD)
            .unwrap();
    let public_key = builder.get_public_key();
    Json(VapidPublicKeyResponse {
        public_key: BASE64_URL_SAFE_NO_PAD.encode(&public_key),
    })
}

// subscribe — POST /push/subscribe
// Legt eine neue Subscription an oder aktualisiert eine bestehende (anhand endpoint). Nutze INSERT OR REPLACE.
// Generiere eine neue UUID für id, speichere created_at als ISO-8601 String (via chrono::Utc::now()).
// Gibt StatusCode::NO_CONTENT zurück.
pub async fn subscribe(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<SubscribeRequest>,
) -> Result<StatusCode, AppError> {
    let uuid = Uuid::now_v7();
    let uuid = uuid.to_string();
    let created_at = chrono::Utc::now();
    sqlx::query!(
        r#"INSERT OR IGNORE INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, created_at) VALUES (?, ?, ?, ?, ?, ?);"#,
       uuid, user_id, body.endpoint, body.p256dh, body.auth, created_at
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// unsubscribe — DELETE /push/subscribe
// Erwartet nur { endpoint: String } im Body. Löscht die passende Subscription für diesen User.
// Gibt NO_CONTENT zurück, auch wenn nichts gelöscht wurde (idempotent).
pub async fn unsubscribe(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<DeletePushRequest>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        r#"DELETE FROM push_subscriptions WHERE endpoint=? AND user_id=?"#,
        body.endpoint,
        user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// update_settings — PUT /push/settings
// Nutze INSERT OR REPLACE INTO user_notification_settings. Gibt NO_CONTENT zurück.

pub async fn update_settings(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Timezone(tz): Timezone,
    Json(body): Json<NotificationSettingsRequest>,
) -> Result<StatusCode, AppError> {
    let timezone = tz.name();
    sqlx::query!(
        r#"INSERT OR REPLACE INTO user_notification_settings (user_id, timezone, notification_hour, notification_days) VALUES (?, ?, ?, ?);"#,
        user_id,
        timezone,
        body.notification_hour,
        body.notification_days
    ).execute(&state.pool).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_settings(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<NotificationSettingsResponse>, AppError> {
    let r = sqlx::query!(
        r#"SELECT notification_hour, notification_days FROM user_notification_settings WHERE user_id=?;"#,
        user_id
    ).fetch_optional(&state.pool).await?;
    Ok(Json(
        r.map(|settings| NotificationSettingsResponse {
            notification_hour: settings.notification_hour,
            notification_days: settings.notification_days,
        })
        .unwrap_or_default(),
    ))
}

#[cfg(debug_assertions)]
pub async fn test_push(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<StatusCode, AppError> {
    use crate::scheduler::notify_user;

    let client = web_push::IsahcWebPushClient::new().map_err(|_| AppError::InternalError)?;
    let now = chrono::Utc::now().date_naive();
    let _ = notify_user(&state, &client, &user_id, now).await;
    Ok(StatusCode::NO_CONTENT)
}
