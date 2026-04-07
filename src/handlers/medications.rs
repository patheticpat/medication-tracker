use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    extractors::LocalDate,
    models::{
        CreateMedication, DbLogEntry, DbMedication, LogEntry, Medication, MedicationWithStats,
        UpdateMedication,
    },
};

pub async fn health() -> &'static str {
    "ok"
}

async fn get_medication_with_logs(pool: &SqlitePool, id: &str) -> Result<Medication, AppError> {
    if let Some(medication) =
        sqlx::query_as!(DbMedication, "SELECT * from medications WHERE id=?", id)
            .fetch_optional(pool)
            .await?
    {
        let medication: Medication = medication.try_into()?;
        let logs = sqlx::query_as!(
            DbLogEntry,
            r#"SELECT kind, amount, date AS "date: NaiveDate", note FROM log_entries WHERE medication_id=? ORDER BY date"#,
            id
        )
        .fetch_all(pool)
        .await?.into_iter().map(|l| l.try_into()).collect::<Result<Vec<LogEntry>, _>>()?;
        Ok(Medication {
            logs: Some(logs),
            ..medication
        })
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn list_medications(
    State(state): State<AppState>,
    LocalDate(at): LocalDate,
) -> Result<Json<Vec<MedicationWithStats>>, AppError> {
    let db_medications = sqlx::query_as!(DbMedication, "SELECT * from medications")
        .fetch_all(&state.pool)
        .await?;

    let mut medications = Vec::with_capacity(db_medications.len());

    for m in db_medications {
        let medication = get_medication_with_logs(&state.pool, m.id.as_ref().unwrap()).await?;
        let stock = medication.calculate_stock(&at);
        let days_remaining = medication.calculate_days_remaining(&at);
        medications.push(MedicationWithStats {
            medication,
            stock,
            days_remaining,
        })
    }
    Ok(Json(medications))
}

pub async fn create_medication(
    State(state): State<AppState>,
    LocalDate(at): LocalDate,
    Json(body): Json<CreateMedication>,
) -> Result<Response, AppError> {
    let id = Uuid::new_v4();
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
        "INSERT INTO medications (id, name, unit, schedule_kind, schedule_amount, schedule_day_of_week, warning_threshold) VALUES (?, ?, ?, ?, ?, ?, ?)",
        id_str,
        body.name,
        body.unit,
        kind,
        amount,
        day,
        body.warning_threshold
    )
    .execute(&mut *tx)
    .await?;
    let log_id = Uuid::new_v4().to_string();
    sqlx::query!("INSERT INTO log_entries (id, medication_id, kind, amount, date, note) VALUES (?, ?, ?, ?, ?, ?)", log_id, id_str, "baseline", body.initial_stock, at, Some("Initial baseline")).execute(&mut *tx).await?;
    tx.commit().await?;

    let medication = get_medication_with_logs(&state.pool, &id_str).await?;
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
    LocalDate(at): LocalDate,
    Path(id): Path<Uuid>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let id = id.to_string();
    let medication = get_medication_with_logs(&state.pool, &id).await?;
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
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let id = id.to_string();

    sqlx::query!("DELETE FROM log_entries WHERE medication_id=?", id)
        .execute(&state.pool)
        .await?;
    let result = sqlx::query!("DELETE FROM medications WHERE id=?", id)
        .execute(&state.pool)
        .await?;
    match result.rows_affected() {
        0 => Err(AppError::NotFound),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

pub async fn update_medication(
    State(state): State<AppState>,
    LocalDate(at): LocalDate,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMedication>,
) -> Result<Json<MedicationWithStats>, AppError> {
    let id = id.to_string();
    if let Some(mut medication) =
        sqlx::query_as!(DbMedication, "SELECT * FROM medications WHERE id=?", id)
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
            medication.warning_threshold = warning_threshold as i64;
        }
        sqlx::query!(
            "UPDATE medications SET name=?, unit=?, schedule_kind=?, schedule_amount=?, schedule_day_of_week=?, warning_threshold=? WHERE id=?",
            medication.name,
            medication.unit,
            medication.schedule_kind,
            medication.schedule_amount,
            medication.schedule_day_of_week,
            medication.warning_threshold,
            id
        ).execute(&state.pool).await?;
        let medication =
            get_medication_with_logs(&state.pool, medication.id.as_ref().unwrap()).await?;
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

pub async fn create_log_entry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<LogEntry>,
) -> Result<Json<Medication>, AppError> {
    let med_id = id.to_string();
    let result = sqlx::query!(
        r#"SELECT COUNT(*) AS count FROM medications WHERE id=?"#,
        med_id
    )
    .fetch_one(&state.pool)
    .await?;
    if result.count != 1 {
        return Err(AppError::NotFound);
    }

    let (kind, amount, date, note) = match body {
        LogEntry::Baseline { amount, date, note } => ("baseline", amount, date, note),
        LogEntry::Refill { amount, date, note } => ("refill", amount, date, note),
    };
    let log_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r"INSERT INTO log_entries (id, medication_id, kind, amount, date, note) VALUES (?, ?, ?, ?, ?, ?)",
        log_id,
        med_id,
        kind,
        amount,
        date,
        note
    ).execute(&state.pool).await?;

    let medication = get_medication_with_logs(&state.pool, &med_id).await?;
    Ok(Json(medication))
}
