use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::AppState;
use crate::errors::AppError;
use crate::models::Claims;

pub struct AuthUser(pub String); // user_id

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Authorization Header extrahieren
        let token = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // 2. JWT validieren
        let key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let token_data = decode::<Claims>(
            &token.token(),
            &key,
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|_| AppError::Unauthorized)?;

        // 3. user_id aus Claims zurückgeben
        Ok(Self(token_data.claims.sub))
    }
}
