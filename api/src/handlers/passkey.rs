use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, Passkey, PublicKeyCredential,
    RegisterPublicKeyCredential, RequestChallengeResponse,
};

use crate::{
    AppState,
    errors::AppError,
    middleware::AuthUser,
    models::{AuthResponse, PasskeyInfo, PasskeyLoginRequest},
    utils::create_jwt,
};

pub async fn register_begin(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<CreationChallengeResponse>, AppError> {
    // 0. Alte Challenges aufräumen
    sqlx::query!(
        r#"DELETE FROM passkey_challenges WHERE timestamp < unixepoch('now', '-300 seconds');"#
    )
    .execute(&state.pool)
    .await?;

    // 1. User aus DB laden
    // 2. Bestehende Credentials des Users laden (damit der Browser sie ausschließen kann)
    let credentials = sqlx::query!(
        r#"SELECT users.*, credentials.credential_id FROM users LEFT JOIN credentials ON users.id=credentials.user_id WHERE users.id=?"#,
        user_id
    ).fetch_all(&state.pool).await?;
    let user_unique_id = uuid::Uuid::parse_str(&user_id).expect("invalid user id format");
    let user = credentials.first().expect("user exists");
    let user_name = user.username.clone();

    // 3. webauthn.start_passkey_registration() aufrufen
    let exclude_credentials = credentials
        .into_iter()
        .filter_map(|row| row.credential_id)
        .filter_map(|id| BASE64_URL_SAFE_NO_PAD.decode(id).ok())
        .map(CredentialID::from)
        .collect();

    let (ccr, skr) = state
        .webauthn
        .start_passkey_registration(
            user_unique_id,
            &user_name,
            &user_name,
            Some(exclude_credentials),
        )
        .map_err(|_| AppError::InternalError)?;

    // 4. Challenge in DB speichern
    let skr = serde_json::to_string(&skr).map_err(|_| AppError::InternalError)?;
    sqlx::query!(
        r#"INSERT OR REPLACE INTO passkey_challenges (user_id, challenge, timestamp) VALUES(?, ?, unixepoch('now'))"#,
        user_id,
        skr
    )
    .execute(&state.pool)
    .await?;

    // 5. CreationChallenge zurückgeben
    Ok(Json(ccr))
}

pub async fn register_complete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<RegisterPublicKeyCredential>,
) -> Result<StatusCode, AppError> {
    // 1. Challenge aus DB laden und löschen
    let challenge = sqlx::query!(
        r#"SELECT * FROM passkey_challenges WHERE user_id=? AND (unixepoch() - timestamp) < 300;"#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    sqlx::query!(r#"DELETE FROM passkey_challenges WHERE user_id=?"#, user_id)
        .execute(&state.pool)
        .await?;
    let skr = serde_json::from_str(&challenge.challenge).map_err(|_| AppError::InternalError)?;

    // 2. webauthn.finish_passkey_registration() aufrufen
    let passkey = state
        .webauthn
        .finish_passkey_registration(&body, &skr)
        .map_err(|_| AppError::Unauthorized)?;

    // 3. Passkey in credentials Tabelle speichern
    let credential_id = BASE64_URL_SAFE_NO_PAD.encode(passkey.cred_id());
    let serialized_passkey =
        serde_json::to_string(&passkey).map_err(|_| AppError::InternalError)?;

    sqlx::query!(r#"INSERT INTO credentials (credential_id, user_id, passkey, added_at) VALUES (?, ?, ?, unixepoch());"#, credential_id, user_id, serialized_passkey).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn login_begin(
    State(state): State<AppState>,
    Json(body): Json<PasskeyLoginRequest>, // nur username
) -> Result<Json<RequestChallengeResponse>, AppError> {
    // 0. Alte Challenges aufräumen
    sqlx::query!(
        r#"DELETE FROM passkey_challenges WHERE timestamp < unixepoch('now', '-300 seconds');"#
    )
    .execute(&state.pool)
    .await?;

    // 1. User aus DB laden
    // 2. Credentials des Users laden
    let credentials = sqlx::query!(
        r#"SELECT users.id as "id!", credentials.passkey FROM users JOIN credentials ON users.id=credentials.user_id WHERE users.username=?"#,
        body.username
    ).fetch_all(&state.pool).await?;

    // Wenn credentials leer ist existiert entweder der User nicht oder er hat keinen Passkey.
    if credentials.is_empty() {
        return Err(AppError::NotFound);
    }

    let creds: Option<Vec<Passkey>> = credentials
        .iter()
        .map(|row| serde_json::from_str(&row.passkey).ok())
        .collect();
    let creds = creds.ok_or(AppError::InternalError)?;

    // 3. webauthn.start_passkey_authentication() aufrufen
    let (rcr, pka) = state
        .webauthn
        .start_passkey_authentication(&creds)
        .map_err(|_| AppError::InternalError)?;

    // 4. Challenge in DB speichern
    let user_id = &credentials.first().expect("credentials is not empty").id;
    let challenge = serde_json::to_string(&pka).map_err(|_| AppError::InternalError)?;
    sqlx::query!(
        r#"INSERT OR REPLACE INTO passkey_challenges (user_id, challenge, timestamp) VALUES(?, ?, unixepoch('now'))"#,
        user_id,
        challenge
    )
    .execute(&state.pool)
    .await?;

    // 5. RequestChallengeResponse zurückgeben
    Ok(Json(rcr))
}

pub async fn login_complete(
    State(state): State<AppState>,
    Json(body): Json<PublicKeyCredential>,
) -> Result<Json<AuthResponse>, AppError> {
    // Aber: woher wissen wir hier welcher User sich anmeldet?
    let credential_id = BASE64_URL_SAFE_NO_PAD.encode(body.raw_id.as_ref());
    let row = sqlx::query!(
        r#"SELECT users.id AS "id!", credentials.passkey FROM users JOIN credentials ON users.id=credentials.user_id WHERE credential_id = ?"#,
        credential_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let user_id = &row.id;
    let mut passkey: Passkey =
        serde_json::from_str(&row.passkey).map_err(|_| AppError::InternalError)?;

    let challenge = sqlx::query!(
        r#"SELECT * FROM passkey_challenges WHERE user_id=? AND (unixepoch() - timestamp) < 300;"#,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query!(
        r#"DELETE FROM passkey_challenges WHERE user_id=?;"#,
        user_id
    )
    .execute(&state.pool)
    .await
    .map_err(|_| AppError::InternalError)?;

    let auth_state =
        serde_json::from_str(&challenge.challenge).map_err(|_| AppError::InternalError)?;
    let authentication_result = state
        .webauthn
        .finish_passkey_authentication(&body, &auth_state)
        .map_err(|_| AppError::Unauthorized)?;

    /*
    As per https://www.w3.org/TR/webauthn-3/#sctn-verifying-assertion 21:

    If the Credential Counter is greater than 0 you MUST assert that the counter is greater than the stored counter.
    If the counter is equal or less than this MAY indicate a cloned credential and you SHOULD invalidate and reject
    that credential as a result.

    From this AuthenticationResult you should update the Credential’s Counter value if it is valid per the above check.
    If you wish you may use the content of the AuthenticationResult for extended validations (such as the presence of
    the user verification flag).
    */

    if authentication_result.needs_update()
        && let Some(true) = passkey.update_credential(&authentication_result)
    {
        let serialized_passkey =
            serde_json::to_string(&passkey).map_err(|_| AppError::InternalError)?;
        sqlx::query!(
            r#"UPDATE credentials SET passkey=?, last_used_at=unixepoch() WHERE credential_id=?"#,
            serialized_passkey,
            credential_id
        )
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query!(
            r#"UPDATE credentials SET last_used_at=unixepoch() WHERE credential_id=?"#,
            credential_id
        )
        .execute(&state.pool)
        .await?;
    }

    // At this point we are ready to create and return an auth token for the user
    let token = create_jwt(user_id, &state.jwt_secret)?;
    Ok(Json(AuthResponse { token }))
}

pub async fn list_passkeys(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<PasskeyInfo>>, AppError> {
    let passkeys = sqlx::query_as!(
        PasskeyInfo,
        r#"SELECT credential_id AS "credential_id!", added_at, last_used_at FROM credentials WHERE user_id=?"#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(passkeys))
}

pub async fn delete_passkey(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(credential_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let r = sqlx::query!(
        r#"DELETE FROM credentials WHERE credential_id=? and user_id=?"#,
        credential_id,
        user_id
    )
    .execute(&state.pool)
    .await?;
    if r.rows_affected() != 1 {
        Err(AppError::NotFound)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
