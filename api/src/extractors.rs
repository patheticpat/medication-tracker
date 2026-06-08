use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::NaiveDate;

pub struct Timezone(pub chrono_tz::Tz);

impl<S> FromRequestParts<S> for Timezone
where
    S: Send + Sync,
{
    type Rejection = Infallible;

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

pub struct LocalDate(pub NaiveDate);

impl LocalDate {
    fn from_timezone_and_time(tz: chrono_tz::Tz, now: chrono::DateTime<chrono::Utc>) -> Self {
        LocalDate(now.with_timezone(&tz).date_naive())
    }
}

impl<S> FromRequestParts<S> for LocalDate
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Timezone(tz) = Timezone::from_request_parts(parts, state).await?;
        Ok(Self::from_timezone_and_time(tz, chrono::Utc::now()))
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    #[tokio::test]
    async fn test_extract_valid_timezone() {
        let request = Request::builder()
            .header("x-timezone", "Asia/Tokyo")
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = Timezone::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok_and(|Timezone(tz)| tz.eq(&chrono_tz::Asia::Tokyo)));
    }

    #[tokio::test]
    async fn test_extract_missing_timezone() {
        let request = Request::builder().body(()).unwrap();

        let (mut parts, _) = request.into_parts();
        let result = Timezone::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok_and(|Timezone(tz)| tz.eq(&chrono_tz::Europe::Berlin)));
    }

    #[tokio::test]
    async fn test_extract_invalid_timezone() {
        let request = Request::builder()
            .header("x-timezone", "Invalid/Timezone")
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = Timezone::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok_and(|Timezone(tz)| tz.eq(&chrono_tz::Europe::Berlin)));
    }

    #[test]
    fn test_local_date_uses_timezone() {
        // 2024-01-15 02:00 UTC = 2024-01-14 in New York (UTC-5)
        let now = chrono::DateTime::parse_from_rfc3339("2024-01-15T02:00:00Z")
            .unwrap()
            .to_utc();

        let date = LocalDate::from_timezone_and_time(chrono_tz::America::New_York, now);
        assert_eq!(
            date.0,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 14).unwrap()
        );
    }
}
