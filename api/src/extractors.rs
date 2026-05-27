use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::{NaiveDate, TimeZone};

use crate::errors::AppError;

pub struct LocalDate(pub NaiveDate);

impl<S> FromRequestParts<S> for LocalDate
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tz: chrono_tz::Tz = parts
            .headers
            .get("x-timezone")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(chrono_tz::Europe::Berlin);

        let date = tz
            .from_utc_datetime(&chrono::Utc::now().naive_utc())
            .date_naive();
        Ok(LocalDate(date))
    }
}

pub struct Timezone(pub chrono_tz::Tz);

impl<S> FromRequestParts<S> for Timezone
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tz: chrono_tz::Tz = parts
            .headers
            .get("x-timezone")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(chrono_tz::Europe::Berlin);

        Ok(Timezone(tz))
    }
}
