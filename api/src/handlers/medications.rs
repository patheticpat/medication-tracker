use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_valid::Valid;
use chrono::NaiveDate;
use color_eyre::eyre::OptionExt;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    extractors::LocalDate,
    middleware::AuthUser,
    models::{
        CreateLogEntryRequest, CreateMedicationRequest, DbMedication, DbMedicationWithLogRow,
        LogEntry, Medication, MedicationWithStats, PatchSnoozeRequest, Schedule,
        UpdateMedicationRequest,
    },
};

fn schedule_from_row(row: &DbMedicationWithLogRow) -> Result<Schedule, AppError> {
    match row.schedule_kind.as_str() {
        "daily" => Ok(Schedule::Daily {
            amount: row.schedule_amount,
        }),
        "weekly" => Ok(Schedule::Weekly {
            day_of_week: row
                .schedule_day_of_week
                .ok_or_eyre("missing day_of_week")?
                .try_into()
                .map_err(|_| AppError::InternalError)?,
            amount: row.schedule_amount,
        }),
        _ => Err(AppError::InternalError),
    }
}

fn medication_from_row(row: &DbMedicationWithLogRow) -> Result<Medication, AppError> {
    Ok(Medication {
        id: Uuid::parse_str(&row.id).map_err(|_| AppError::InternalError)?,
        name: row.name.clone(),
        unit: row.unit.clone(),
        unit_singular: row.unit_singular.clone(),
        schedule: schedule_from_row(row)?,
        warning_threshold: row
            .warning_threshold
            .try_into()
            .map_err(|_| AppError::InternalError)?,
        snoozed: row.snoozed,
        logs: Some(Vec::new()),
    })
}

fn log_entry_from_row(row: DbMedicationWithLogRow) -> Result<Option<LogEntry>, AppError> {
    let (Some(kind), Some(amount), Some(date)) = (row.log_kind, row.log_amount, row.log_date)
    else {
        return Ok(None);
    };
    let id = Uuid::parse_str(row.log_id.ok_or_eyre("missing log id")?.as_str())
        .map_err(|_| AppError::InternalError)?;
    let log = match kind.as_str() {
        "baseline" => LogEntry::Baseline {
            id,
            amount,
            date,
            note: row.log_note,
        },
        "refill" => LogEntry::Refill {
            id,
            amount,
            date,
            note: row.log_note,
        },
        _ => return Err(AppError::InternalError),
    };
    Ok(Some(log))
}

pub async fn get_all_medications_with_logs(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<Medication>, AppError> {
    let rows = sqlx::query_as!(
        DbMedicationWithLogRow,
        r#"
        SELECT
            m.id AS 'id!',
            m.user_id,
            m.name,
            m.unit,
            m.unit_singular,
            m.schedule_kind,
            m.schedule_amount,
            m.schedule_day_of_week,
            m.warning_threshold,
            m.snoozed,
            l.id AS log_id, l.kind AS log_kind, l.amount AS log_amount,
            l.date AS "log_date: NaiveDate", l.note AS log_note
        FROM medications m LEFT JOIN log_entries l ON m.id = l.medication_id
        WHERE m.user_id = ?
        ORDER BY m.id, l.date, l.id
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut medications: Vec<Medication> = Vec::new();

    for row in rows {
        if medications
            .last()
            .map_or(true, |m| m.id.to_string() != row.id)
        {
            medications.push(medication_from_row(&row)?);
        }
        if let Some(log) = log_entry_from_row(row)? {
            medications
                .last_mut()
                .unwrap()
                .logs
                .as_mut()
                .unwrap()
                .push(log);
        }
    }

    Ok(medications)
}

async fn get_medication_with_logs(
    pool: &SqlitePool,
    medication_id: &str,
    user_id: &str,
) -> Result<Medication, AppError> {
    let rows = sqlx::query_as!(
        DbMedicationWithLogRow,
        r#"
        SELECT
            m.id AS 'id!',
            m.user_id,
            m.name,
            m.unit,
            m.unit_singular,
            m.schedule_kind,
            m.schedule_amount,
            m.schedule_day_of_week,
            m.warning_threshold,
            m.snoozed,
            l.id AS log_id, l.kind AS log_kind, l.amount AS log_amount,
            l.date AS "log_date: NaiveDate", l.note AS log_note
        FROM medications m LEFT JOIN log_entries l ON m.id = l.medication_id
        WHERE m.user_id = ? AND m.id = ?
        ORDER BY m.id, l.date, l.id
        "#,
        user_id,
        medication_id
    )
    .fetch_all(pool)
    .await?;

    let mut rows = rows.into_iter();
    let first = rows.next().ok_or(AppError::NotFound)?;
    let mut medication = medication_from_row(&first)?;

    let logs = std::iter::once(first)
        .chain(rows)
        .filter_map(|row| log_entry_from_row(row).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    medication.logs = Some(logs);
    Ok(medication)
}

pub async fn list_medications(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    LocalDate(at): LocalDate,
) -> Result<Json<Vec<MedicationWithStats>>, AppError> {
    let medications = get_all_medications_with_logs(&state.pool, &user_id).await?;

    let medications_with_stats = medications
        .into_iter()
        .map(|medication| {
            let stock = medication.calculate_stock(&at);
            let days_remaining = medication.calculate_days_remaining(&at);
            MedicationWithStats {
                medication: Medication {
                    logs: None,
                    ..medication
                },
                stock,
                days_remaining,
            }
        })
        .collect();
    Ok(Json(medications_with_stats))
}

pub async fn create_medication(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    LocalDate(at): LocalDate,
    Valid(Json(body)): Valid<Json<CreateMedicationRequest>>,
) -> Result<Response, AppError> {
    let id = Uuid::now_v7();
    let id_str = id.to_string();
    let (kind, amount, day) = match body.schedule {
        crate::models::Schedule::Daily { amount } => ("daily", amount, None),
        crate::models::Schedule::Weekly {
            day_of_week,
            amount,
        } => ("weekly", amount, Some(day_of_week)),
    };

    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        r"
        INSERT INTO medications (
            id,
            user_id,
            name,
            unit,
            unit_singular,
            schedule_kind,
            schedule_amount,
            schedule_day_of_week,
            warning_threshold
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ",
        id_str,
        user_id,
        body.name,
        body.unit,
        body.unit_singular,
        kind,
        amount,
        day,
        body.warning_threshold
    )
    .execute(&mut *tx)
    .await?;
    let log_id = Uuid::now_v7().to_string();
    sqlx::query!(
        r"
    INSERT INTO log_entries (id, medication_id, kind, amount, date, note) VALUES (
        ?, ?, ?, ?, ?, ?
    )
    ",
        log_id,
        id_str,
        "baseline",
        body.initial_stock,
        at,
        body.initial_note
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let medication = get_medication_with_logs(&state.pool, &id_str, &user_id).await?;
    let stock = medication.calculate_stock(&at);
    let days_remaining = medication.calculate_days_remaining(&at);

    Ok((
        StatusCode::CREATED,
        Json(MedicationWithStats {
            medication,
            stock,
            days_remaining,
        }),
    )
        .into_response())
}

pub async fn medication_details(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    LocalDate(at): LocalDate,
    Path(id): Path<Uuid>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let id = id.to_string();
    let medication = get_medication_with_logs(&state.pool, &id, &user_id).await?;
    let stock = medication.calculate_stock(&at);
    let days_remaining = medication.calculate_days_remaining(&at);
    Ok(Json(MedicationWithStats {
        medication,
        stock,
        days_remaining,
    }))
}

pub async fn delete_medication(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let id = id.to_string();

    let result = sqlx::query!(
        "DELETE FROM medications WHERE id = ? AND user_id = ?",
        id,
        user_id
    )
    .execute(&state.pool)
    .await?;
    match result.rows_affected() {
        0 => Err(AppError::NotFound),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

pub async fn update_medication(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    LocalDate(at): LocalDate,
    Path(id): Path<Uuid>,
    Valid(Json(body)): Valid<Json<UpdateMedicationRequest>>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let id = id.to_string();
    if let Some(mut medication) = sqlx::query_as!(
        DbMedication,
        r"
        SELECT
            id AS 'id!',
            user_id,
            name,
            unit,
            unit_singular,
            schedule_kind,
            schedule_amount,
            schedule_day_of_week,
            warning_threshold,
            snoozed
        FROM medications
        WHERE id = ? AND user_id = ?
        ",
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    {
        if let Some(name) = body.name {
            medication.name = name;
        }
        if let Some(unit) = body.unit {
            medication.unit = unit;
        }
        if let Some(schedule) = body.schedule {
            let (kind, amount, day) = match schedule {
                crate::models::Schedule::Daily { amount } => (String::from("daily"), amount, None),
                crate::models::Schedule::Weekly {
                    day_of_week,
                    amount,
                } => (String::from("weekly"), amount, Some(day_of_week as i64)),
            };

            medication.schedule_kind = kind;
            medication.schedule_amount = amount;
            medication.schedule_day_of_week = day;
        }
        if let Some(warning_threshold) = body.warning_threshold {
            medication.snoozed = if warning_threshold as i64 == medication.warning_threshold {
                medication.snoozed
            } else {
                false
            };
            medication.warning_threshold = warning_threshold as i64;
        }
        sqlx::query!(
            r"
            UPDATE medications SET name = ?,
            unit = ?,
            unit_singular = ?,
            schedule_kind = ?,
            schedule_amount = ?,
            schedule_day_of_week = ?,
            warning_threshold = ?,
            snoozed = ? WHERE id = ? AND user_id = ?
            ",
            medication.name,
            medication.unit,
            body.unit_singular,
            medication.schedule_kind,
            medication.schedule_amount,
            medication.schedule_day_of_week,
            medication.warning_threshold,
            medication.snoozed,
            id,
            user_id
        )
        .execute(&state.pool)
        .await?;
        let medication = get_medication_with_logs(&state.pool, &medication.id, &user_id).await?;
        let stock = medication.calculate_stock(&at);
        let days_remaining = medication.calculate_days_remaining(&at);
        Ok(Json(MedicationWithStats {
            medication,
            stock,
            days_remaining,
        }))
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn patch_snooze(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    LocalDate(at): LocalDate,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchSnoozeRequest>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let id = id.to_string();
    let result = sqlx::query!(
        "UPDATE medications SET snoozed = ? WHERE id = ? AND user_id = ?",
        body.snoozed,
        id,
        user_id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() != 1 {
        return Err(AppError::NotFound);
    }

    let medication = get_medication_with_logs(&state.pool, &id, &user_id).await?;
    let stock = medication.calculate_stock(&at);
    let days_remaining = medication.calculate_days_remaining(&at);
    Ok(Json(MedicationWithStats {
        medication,
        stock,
        days_remaining,
    }))
}

pub async fn create_log_entry(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<Uuid>,
    LocalDate(today): LocalDate,
    Valid(Json(body)): Valid<Json<CreateLogEntryRequest>>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let med_id = id.to_string();
    let result = sqlx::query!(
        "SELECT COUNT(*) AS count FROM medications WHERE id = ? AND user_id = ?",
        med_id,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;
    if result.count != 1 {
        return Err(AppError::NotFound);
    }

    let (kind, amount, note) = match body {
        CreateLogEntryRequest::Baseline { amount, note } => ("baseline", amount, note),
        CreateLogEntryRequest::Refill { amount, note } => ("refill", amount, note),
    };
    let log_id = Uuid::now_v7().to_string();

    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        r"
        INSERT INTO log_entries (id, medication_id, kind, amount, date, note) VALUES (
            ?, ?, ?, ?, ?, ?
        )
        ",
        log_id,
        med_id,
        kind,
        amount,
        today,
        note
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE medications SET snoozed = FALSE WHERE id = ?",
        med_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let medication = get_medication_with_logs(&state.pool, &med_id, &user_id).await?;
    let stock = medication.calculate_stock(&today);
    let days_remaining = medication.calculate_days_remaining(&today);
    Ok(Json(MedicationWithStats {
        medication,
        stock,
        days_remaining,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn base_row() -> DbMedicationWithLogRow {
        DbMedicationWithLogRow {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            user_id: "user1".to_string(),
            name: "Aspirin".to_string(),
            unit: "tablets".to_string(),
            unit_singular: Some("tablet".to_string()),
            schedule_kind: "daily".to_string(),
            schedule_amount: 1.0,
            schedule_day_of_week: None,
            warning_threshold: 5,
            snoozed: false,
            log_id: None,
            log_kind: None,
            log_amount: None,
            log_date: None,
            log_note: None,
        }
    }

    // --- schedule_from_row ---

    #[test]
    fn schedule_daily() {
        let schedule = schedule_from_row(&base_row()).unwrap();
        assert!(matches!(schedule, Schedule::Daily { amount } if amount == 1.0));
    }

    #[test]
    fn schedule_weekly() {
        let row = DbMedicationWithLogRow {
            schedule_kind: "weekly".to_string(),
            schedule_amount: 2.0,
            schedule_day_of_week: Some(3),
            ..base_row()
        };
        let schedule = schedule_from_row(&row).unwrap();
        assert!(matches!(
            schedule,
            Schedule::Weekly { day_of_week: 3, amount } if amount == 2.0
        ));
    }

    #[test]
    fn schedule_weekly_missing_day_of_week_errors() {
        let row = DbMedicationWithLogRow {
            schedule_kind: "weekly".to_string(),
            schedule_day_of_week: None,
            ..base_row()
        };
        assert!(schedule_from_row(&row).is_err());
    }

    #[test]
    fn schedule_unknown_kind_errors() {
        let row = DbMedicationWithLogRow {
            schedule_kind: "monthly".to_string(),
            ..base_row()
        };
        assert!(schedule_from_row(&row).is_err());
    }

    // --- medication_from_row ---

    #[test]
    fn medication_from_valid_daily_row() {
        let med = medication_from_row(&base_row()).unwrap();
        assert_eq!(med.name, "Aspirin");
        assert_eq!(med.unit, "tablets");
        assert_eq!(med.unit_singular, Some("tablet".to_string()));
        assert_eq!(med.warning_threshold, 5);
        assert!(!med.snoozed);
        assert!(matches!(med.logs, Some(ref v) if v.is_empty()));
    }

    #[test]
    fn medication_from_valid_weekly_row() {
        let row = DbMedicationWithLogRow {
            schedule_kind: "weekly".to_string(),
            schedule_day_of_week: Some(1),
            ..base_row()
        };
        let med = medication_from_row(&row).unwrap();
        assert!(matches!(
            med.schedule,
            Schedule::Weekly { day_of_week: 1, .. }
        ));
    }

    #[test]
    fn medication_from_row_invalid_uuid_errors() {
        let row = DbMedicationWithLogRow {
            id: "not-a-uuid".to_string(),
            ..base_row()
        };
        assert!(medication_from_row(&row).is_err());
    }

    #[test]
    fn medication_from_row_negative_warning_threshold_errors() {
        let row = DbMedicationWithLogRow {
            warning_threshold: -1,
            ..base_row()
        };
        assert!(medication_from_row(&row).is_err());
    }

    // --- log_entry_from_row ---

    #[test]
    fn log_entry_no_log_fields_returns_none() {
        assert!(log_entry_from_row(base_row()).unwrap().is_none());
    }

    #[test]
    fn log_entry_partial_log_fields_returns_none() {
        let row = DbMedicationWithLogRow {
            log_kind: Some("baseline".to_string()),
            ..base_row()
        };
        assert!(log_entry_from_row(row).unwrap().is_none());
    }

    #[test]
    fn log_entry_baseline() {
        let log_id = Uuid::now_v7().to_string();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let row = DbMedicationWithLogRow {
            log_id: Some(log_id),
            log_kind: Some("baseline".to_string()),
            log_amount: Some(10.0),
            log_date: Some(date),
            log_note: Some("initial".to_string()),
            ..base_row()
        };
        let log = log_entry_from_row(row).unwrap().unwrap();
        assert!(matches!(
            log,
            LogEntry::Baseline { amount, date: d, note: Some(ref n), .. }
            if amount == 10.0 && d == date && n == "initial"
        ));
    }

    #[test]
    fn log_entry_refill_no_note() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let row = DbMedicationWithLogRow {
            log_id: Some(Uuid::now_v7().to_string()),
            log_kind: Some("refill".to_string()),
            log_amount: Some(30.0),
            log_date: Some(date),
            log_note: None,
            ..base_row()
        };
        let log = log_entry_from_row(row).unwrap().unwrap();
        assert!(matches!(
            log,
            LogEntry::Refill { amount, date: d, note: None, .. }
            if amount == 30.0 && d == date
        ));
    }

    #[test]
    fn log_entry_missing_log_id_errors() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let row = DbMedicationWithLogRow {
            log_id: None,
            log_kind: Some("baseline".to_string()),
            log_amount: Some(5.0),
            log_date: Some(date),
            ..base_row()
        };
        assert!(log_entry_from_row(row).is_err());
    }

    #[test]
    fn log_entry_invalid_log_uuid_errors() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let row = DbMedicationWithLogRow {
            log_id: Some("bad-uuid".to_string()),
            log_kind: Some("baseline".to_string()),
            log_amount: Some(5.0),
            log_date: Some(date),
            ..base_row()
        };
        assert!(log_entry_from_row(row).is_err());
    }

    #[test]
    fn log_entry_unknown_kind_errors() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let row = DbMedicationWithLogRow {
            log_id: Some(Uuid::now_v7().to_string()),
            log_kind: Some("unknown".to_string()),
            log_amount: Some(5.0),
            log_date: Some(date),
            ..base_row()
        };
        assert!(log_entry_from_row(row).is_err());
    }
}
