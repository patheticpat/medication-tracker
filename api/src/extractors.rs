use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use chrono::{NaiveDate, TimeZone};
use chrono_tz::Europe::Berlin;

use crate::{errors::AppError, models::DateQuery};

pub struct LocalDate(pub NaiveDate);

impl<S> FromRequestParts<S> for LocalDate
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let date = if let Ok(Query(params)) =
            Query::<DateQuery>::from_request_parts(parts, _state).await
        {
            params.date.unwrap_or_else(|| {
                Berlin
                    .from_utc_datetime(&chrono::Utc::now().naive_utc())
                    .date_naive()
            })
        } else {
            Berlin
                .from_utc_datetime(&chrono::Utc::now().naive_utc())
                .date_naive()
        };

        Ok(LocalDate(date))
    }
}
