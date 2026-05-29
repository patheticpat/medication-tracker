use std::error::Error;

use chrono::{Datelike, NaiveDate, Timelike};
use web_push::{
    IsahcWebPushClient, SubscriptionInfo, URL_SAFE_NO_PAD, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

use crate::{AppState, handlers::medications::get_all_medications_with_logs};

pub async fn run(state: &AppState) -> Result<(), Box<dyn Error>> {
    let client = IsahcWebPushClient::new()?;
    let timezones = sqlx::query!(r#"SELECT DISTINCT timezone FROM user_notification_settings;"#)
        .fetch_all(&state.pool)
        .await?;

    for row in timezones {
        let tz = row.timezone.parse::<chrono_tz::Tz>()?;
        let local_now = chrono::Utc::now().with_timezone(&tz);
        let (hour, day) = (local_now.hour(), local_now.weekday().num_days_from_sunday());

        let user_rows = sqlx::query!(r#"SELECT user_id AS 'user_id!' FROM user_notification_settings WHERE timezone=? AND notification_hour=? AND notification_days LIKE '%' || ? || '%';"#, row.timezone, hour, day).fetch_all(&state.pool).await?;

        for user in user_rows {
            let _ = notify_user(state, &client, &user.user_id, local_now.date_naive()).await;
        }
    }
    Ok(())
}

pub async fn notify_user<W: WebPushClient>(
    state: &AppState,
    client: &W,
    user_id: &str,
    now: NaiveDate,
) -> Result<(), Box<dyn Error>> {
    let medications = get_all_medications_with_logs(&state.pool, user_id).await?;
    let count = medications
        .iter()
        .filter(|m| !m.snoozed && m.calculate_days_remaining(&now) <= m.warning_threshold as u64)
        .count();
    if count > 0 {
        let subscriptions = sqlx::query!(
            r#"SELECT endpoint, p256dh, auth FROM push_subscriptions WHERE user_id=?"#,
            user_id
        )
        .fetch_all(&state.pool)
        .await?;
        let body = serde_json::json!({
            "title": "Medication Tracker",
            "body": format!("{} Medikament(e) laufen bald ab", count)
        })
        .to_string();
        for subscription in subscriptions {
            let subscription_info = SubscriptionInfo::new(
                subscription.endpoint,
                subscription.p256dh,
                subscription.auth,
            );
            let _ = send_notification(state, client, &subscription_info, body.as_bytes()).await;
        }
    }
    Ok(())
}

async fn send_notification<W: WebPushClient>(
    state: &AppState,
    client: &W,
    subscription_info: &SubscriptionInfo,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sig_builder = VapidSignatureBuilder::from_base64(
        &state.vapid_private_key,
        URL_SAFE_NO_PAD,
        subscription_info,
    )?;
    sig_builder.add_claim::<&str>("sub", state.vapid_subject.as_ref());
    let signature = sig_builder.build()?;
    let mut message = WebPushMessageBuilder::new(subscription_info);
    message.set_payload(web_push::ContentEncoding::Aes128Gcm, content);
    message.set_vapid_signature(signature);
    let message = message.build()?;
    client.send(message).await?;
    Ok(())
}
