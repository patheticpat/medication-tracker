use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{EncodingKey, Header};

use crate::{errors::AppError, models::Claims};

pub(crate) fn create_jwt(user_id: &str, secret: &str) -> Result<String, AppError> {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 30 * 24 * 60 * 60; // 30 Tage
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalError)?;
    Ok(token)
}
