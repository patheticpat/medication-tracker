use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{ChangePasswordRequest, LoginRequest};
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
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e
            && db_err.kind() == sqlx::error::ErrorKind::UniqueViolation
        {
            return AppError::Conflict(String::from("username already taken"));
        }
        AppError::Database(e)
    })?;

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

pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
    // 1. User aus DB laden (SELECT password_hash FROM users WHERE id=?)
    let row = sqlx::query!(r#"SELECT password_hash FROM users WHERE id=?"#, user_id)
        .fetch_one(&state.pool)
        .await?;

    // 2. Aktuelles Passwort mit Argon2 verifizieren → bei Fehler AppError::Unauthorized
    let parsed_hash = PasswordHash::new(&row.password_hash).map_err(|_| AppError::InternalError)?;
    Argon2::default()
        .verify_password(body.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest(String::from("wrong current password")))?;

    // 3. Neues Passwort hashen
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.new_password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalError)?
        .to_string();

    // 4. Hash in DB speichern (UPDATE users SET password_hash=? WHERE id=?)
    let r = sqlx::query!(
        r#"UPDATE users SET password_hash=? WHERE id=?"#,
        password_hash,
        user_id
    )
    .execute(&state.pool)
    .await?;

    if r.rows_affected() != 1 {
        return Err(AppError::InternalError);
    }

    // 5. Ok(StatusCode::NO_CONTENT)
    Ok(StatusCode::NO_CONTENT)
}
