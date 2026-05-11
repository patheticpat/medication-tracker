use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{Json, extract::State};
use uuid::Uuid;

use crate::models::LoginRequest;
use crate::utils::create_jwt;
use crate::{
    AppState,
    errors::AppError,
    models::{AuthResponse, RegisterRequest},
};

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. Passwort hashen mit argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalError)?
        .to_string();

    // 2. User in DB speichern
    let id = Uuid::now_v7().to_string();
    sqlx::query!(
        r#"INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)"#,
        id,
        body.username,
        password_hash,
    )
    .execute(&state.pool)
    .await?;

    // 3. JWT erstellen und zurückgeben
    let token = create_jwt(&id, &state.jwt_secret)?;
    Ok(Json(AuthResponse { token }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if let Some(user) = sqlx::query!(r#"SELECT * FROM users WHERE username=?"#, body.username)
        .fetch_optional(&state.pool)
        .await?
    {
        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::InternalError)?;

        Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized)?;
        let token = create_jwt(user.id.as_ref().unwrap(), &state.jwt_secret)?;
        Ok(Json(AuthResponse { token }))
    } else {
        Err(AppError::Unauthorized)
    }
}
