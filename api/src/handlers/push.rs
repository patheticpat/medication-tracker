use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    extractors::Timezone,
    middleware::AuthUser,
    models::{DeletePushRequest, NotificationSettingsRequest, SubscribeRequest},
};

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
