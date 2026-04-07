use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medication {
    pub id: Uuid,
    pub name: String,
    pub unit: String,
    pub schedule: Schedule,
    pub warning_threshold: u16,
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LogEntry {
    Baseline {
        amount: f64,
        date: NaiveDate,
        note: Option<String>,
    },
    Refill {
        amount: f64,
        date: NaiveDate,
        note: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Schedule {
    Daily { amount: f64 },
    Weekly { day_of_week: u8, amount: f64 },
}
