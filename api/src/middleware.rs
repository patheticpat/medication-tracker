use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_jwt_auth::Claims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::errors::AppError;

pub struct AuthUser(pub String); // user_id

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomClaims {
    sub: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let custom_claims = Claims::<CustomClaims>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let id = Uuid::now_v7();
        let id = id.to_string();

        let mut tx = state.pool.begin().await?;
        sqlx::query!(
            r"
            INSERT OR IGNORE INTO users (id, auth0_sub)
            VALUES (?, ?);
            ",
            id,
            custom_claims.claims.sub
        )
        .execute(&mut *tx)
        .await?;
        let r = sqlx::query!(
            r"
            SELECT id as 'id!' FROM users WHERE auth0_sub = ?;
            ",
            custom_claims.claims.sub
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(Self(r.id))
    }
}
