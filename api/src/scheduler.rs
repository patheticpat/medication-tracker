use std::error::Error;

use chrono::{Datelike, NaiveDate, Timelike};
use web_push::{SubscriptionInfo, WebPushClient, WebPushMessageBuilder};

use crate::{AppState, handlers::medications::get_all_medications_with_logs};

pub async fn run(state: &AppState) -> Result<(), Box<dyn Error>> {
    let timezones = sqlx::query!(
        r"
        SELECT DISTINCT s.timezone FROM user_notification_settings s
        JOIN push_subscriptions p
            ON s.user_id = p.user_id
        "
    )
    .fetch_all(&state.pool)
    .await?;

    for row in timezones {
        let tz = row.timezone.parse::<chrono_tz::Tz>()?;
        let local_now = chrono::Utc::now().with_timezone(&tz);
        let (hour, day) = (local_now.hour(), local_now.weekday().num_days_from_sunday());

        // Load all users that are due at the current time _and_ have at least one active push_subscription
        let user_rows = sqlx::query!(
            r"
            SELECT DISTINCT s.user_id AS 'user_id!'
            FROM user_notification_settings s
            JOIN push_subscriptions p
                ON s.user_id = p.user_id
            WHERE
                timezone
            = ? AND notification_hour = ? AND notification_days LIKE '%' || ? || '%' ;
            ",
            row.timezone,
            hour,
            day,
        )
        .fetch_all(&state.pool)
        .await?;

        for user in user_rows {
            let _ = notify_user(state, &user.user_id, local_now.date_naive()).await;
        }
    }
    Ok(())
}

pub async fn notify_user(
    state: &AppState,
    user_id: &str,
    now: NaiveDate,
) -> Result<(), Box<dyn Error>> {
    let medications = get_all_medications_with_logs(&state.pool, user_id).await?;
    let count = medications
        .into_iter()
        .filter(|m| !m.snoozed && m.calculate_days_remaining(&now) <= m.warning_threshold as u64)
        .count();
    if count > 0 {
        let subscriptions = sqlx::query!(
            "SELECT endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = ?",
            user_id
        )
        .fetch_all(&state.pool)
        .await?;
        let body = serde_json::json!({
            "title": "Medication Tracker",
            "body": if count == 1 {
    "1 Medikament läuft bald ab.".to_string()
} else {
    format!("{} Medikamente laufen bald ab.", count)
}
        })
        .to_string();
        for subscription in subscriptions {
            let subscription_info = SubscriptionInfo::new(
                subscription.endpoint,
                subscription.p256dh,
                subscription.auth,
            );
            let _ = send_notification(state, &subscription_info, body.as_bytes()).await;
        }
    }
    Ok(())
}

pub async fn send_notification(
    state: &AppState,
    subscription_info: &SubscriptionInfo,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let sig_builder = state.vapid_signature_builder.clone();
    let mut sig_builder = sig_builder.add_sub_info(subscription_info);
    sig_builder.add_claim::<&str>("sub", state.vapid_subject.as_ref());
    let signature = sig_builder.build()?;

    let mut message = WebPushMessageBuilder::new(subscription_info);
    message.set_payload(web_push::ContentEncoding::Aes128Gcm, content);
    message.set_vapid_signature(signature);

    let message = message.build()?;
    state.push_client.send(message).await.map_err(Into::into)
}
