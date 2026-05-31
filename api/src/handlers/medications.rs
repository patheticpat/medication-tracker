use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
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
        CreateLogEntry, CreateMedication, DbMedication, DbMedicationWithLogRow, LogEntry,
        Medication, MedicationWithStats, PatchSnooze, Schedule, UpdateMedication,
    },
};

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
    let mut current_id = String::new();

    for row in rows {
        if row.id != current_id {
            current_id = row.id.clone();
            let medication = Medication {
                id: Uuid::parse_str(&row.id).map_err(|_| AppError::InternalError)?,
                name: row.name,
                unit: row.unit,
                schedule: match row.schedule_kind.as_str() {
                    "daily" => Schedule::Daily {
                        amount: row.schedule_amount,
                    },
                    "weekly" => Schedule::Weekly {
                        day_of_week: row
                            .schedule_day_of_week
                            .ok_or_eyre("missing day_of_week")?
                            .try_into()
                            .map_err(|_| AppError::InternalError)?,
                        amount: row.schedule_amount,
                    },
                    _ => return Err(AppError::InternalError),
                },
                warning_threshold: row
                    .warning_threshold
                    .try_into()
                    .map_err(|_| AppError::InternalError)?,
                snoozed: row.snoozed,
                logs: Some(Vec::new()),
            };
            medications.push(medication);
        }

        if let (Some(kind), Some(amount), Some(date)) = (row.log_kind, row.log_amount, row.log_date)
        {
            let id = row.log_id.ok_or_eyre("missing log id")?;
            let id = Uuid::parse_str(&id).map_err(|_| AppError::InternalError)?;
            let note = row.log_note;
            let log = match kind.as_str() {
                "baseline" => LogEntry::Baseline {
                    id,
                    amount,
                    date,
                    note,
                },
                "refill" => LogEntry::Refill {
                    id,
                    amount,
                    date,
                    note,
                },
                _ => return Err(AppError::InternalError),
            };
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
    let count = rows.len();
    let mut rows = rows.into_iter().peekable();

    let medication = if let Some(row) = rows.peek() {
        Medication {
            id: Uuid::parse_str(&row.id).map_err(|_| AppError::InternalError)?,
            name: row.name.clone(),
            unit: row.unit.clone(),
            schedule: match row.schedule_kind.as_str() {
                "daily" => Schedule::Daily {
                    amount: row.schedule_amount,
                },
                "weekly" => Schedule::Weekly {
                    day_of_week: row
                        .schedule_day_of_week
                        .ok_or_eyre("missing day_of_week")?
                        .try_into()
                        .map_err(|_| AppError::InternalError)?,
                    amount: row.schedule_amount,
                },
                _ => return Err(AppError::InternalError),
            },
            warning_threshold: row
                .warning_threshold
                .try_into()
                .map_err(|_| AppError::InternalError)?,
            snoozed: row.snoozed,
            logs: None,
        }
    } else {
        return Err(AppError::NotFound);
    };
    let mut logs = Vec::with_capacity(count);

    for row in rows {
        if let (Some(kind), Some(amount), Some(date)) = (row.log_kind, row.log_amount, row.log_date)
        {
            let id = row.log_id.ok_or_eyre("missing log id")?;
            let id = Uuid::parse_str(&id).map_err(|_| AppError::InternalError)?;
            let note = row.log_note;
            let log = match kind.as_str() {
                "baseline" => LogEntry::Baseline {
                    id,
                    amount,
                    date,
                    note,
                },
                "refill" => LogEntry::Refill {
                    id,
                    amount,
                    date,
                    note,
                },
                _ => return Err(AppError::InternalError),
            };
            logs.push(log);
        }
    }
    Ok(Medication {
        logs: Some(logs),
        ..medication
    })
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
    Json(body): Json<CreateMedication>,
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

    if amount <= 0. {
        return Err(AppError::BadRequest(String::from(
            "Schedule must have a strictly positive amount",
        )));
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        r"
        INSERT INTO medications (
            id,
            user_id,
            name,
            unit,
            schedule_kind,
            schedule_amount,
            schedule_day_of_week,
            warning_threshold
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ",
        id_str,
        user_id,
        body.name,
        body.unit,
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
        Some("Initial baseline")
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
    Json(body): Json<UpdateMedication>,
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

            if amount <= 0. {
                return Err(AppError::BadRequest(String::from(
                    "Schedule must have a strictly positive amount",
                )));
            }
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
            schedule_kind = ?,
            schedule_amount = ?,
            schedule_day_of_week = ?,
            warning_threshold = ?,
            snoozed = ? WHERE id = ? AND user_id = ?
            ",
            medication.name,
            medication.unit,
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
    Json(body): Json<PatchSnooze>,
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
    Json(body): Json<CreateLogEntry>,
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
        CreateLogEntry::Baseline { amount, note } => ("baseline", amount, note),
        CreateLogEntry::Refill { amount, note } => ("refill", amount, note),
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
