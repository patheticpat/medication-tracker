use chrono::{Datelike, NaiveDate, TimeDelta};
use color_eyre::eyre::eyre;
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
    pub logs: Option<Vec<LogEntry>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MedicationWithStats {
    #[serde(flatten)]
    pub medication: Medication,
    pub stock: f64,
    pub days_remaining: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMedication {
    pub name: String,
    pub unit: String,
    pub schedule: Schedule,
    pub warning_threshold: u16,
    pub initial_stock: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMedication {
    pub name: Option<String>,
    pub unit: Option<String>,
    pub schedule: Option<Schedule>,
    pub warning_threshold: Option<u16>,
}

#[derive(Debug)]
pub struct DbMedication {
    pub id: Option<String>,
    pub name: String,
    pub unit: String,
    pub schedule_kind: String,
    pub schedule_amount: f64,
    pub schedule_day_of_week: Option<i64>,
    pub warning_threshold: i64,
}

impl TryFrom<DbMedication> for Medication {
    type Error = color_eyre::Report;

    fn try_from(value: DbMedication) -> Result<Self, Self::Error> {
        Ok(Medication {
            id: Uuid::parse_str(&value.id.unwrap())?,
            name: value.name,
            unit: value.unit,
            schedule: match value.schedule_kind.as_ref() {
                "daily" => Schedule::Daily {
                    amount: value.schedule_amount,
                },
                "weekly" => Schedule::Weekly {
                    day_of_week: value.schedule_day_of_week.unwrap().try_into()?,
                    amount: value.schedule_amount,
                },
                _ => return Err(eyre!("unknown schedule kind")),
            },
            warning_threshold: value.warning_threshold.try_into()?,
            logs: None,
        })
    }
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

pub struct DbLogEntry {
    pub kind: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub note: Option<String>,
}

impl TryFrom<DbLogEntry> for LogEntry {
    type Error = color_eyre::Report;

    fn try_from(value: DbLogEntry) -> Result<Self, Self::Error> {
        match value.kind.as_ref() {
            "baseline" => Ok(LogEntry::Baseline {
                amount: value.amount,
                date: value.date,
                note: value.note,
            }),
            "refill" => Ok(LogEntry::Refill {
                amount: value.amount,
                date: value.date,
                note: value.note,
            }),
            _ => Err(eyre!("unknown kind")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Schedule {
    Daily { amount: f64 },
    Weekly { day_of_week: u8, amount: f64 },
}
impl LogEntry {
    fn date(&self) -> &NaiveDate {
        match self {
            LogEntry::Baseline { date, .. } => date,
            LogEntry::Refill { date, .. } => date,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateQuery {
    pub date: Option<NaiveDate>,
}

fn count_weekday_between(start: NaiveDate, end: NaiveDate, target_wd: u8) -> i64 {
    if start > end {
        return 0;
    }

    let start_wd = start.weekday().num_days_from_sunday() as i64;
    let offset = (target_wd as i64 - start_wd + 7) % 7;

    let first = start + TimeDelta::days(offset);
    if first > end {
        0
    } else {
        1 + (end - first).num_days() / 7
    }
}

impl Medication {
    fn calculate_consumption(&self, from: &NaiveDate, to: &NaiveDate) -> f64 {
        let from = *from + TimeDelta::days(1);
        let days = (*to - from).num_days() + 1;
        if days < 1 {
            return 0.;
        }
        match self.schedule {
            Schedule::Daily { amount } => days as f64 * amount,
            Schedule::Weekly {
                day_of_week,
                amount,
            } => {
                let days = count_weekday_between(from, *to, day_of_week) as f64;
                days * amount
            }
        }
    }

    pub fn calculate_stock(&self, at: &NaiveDate) -> f64 {
        if let Some(logs) = &self.logs {
            let (anchor_date, stock) = logs.iter().take_while(|log| log.date() <= at).fold(
                (None, 0.),
                |(baseline, stock), log| match log {
                    LogEntry::Baseline { amount, date, .. } => (Some(date), *amount),
                    LogEntry::Refill { amount, .. } => (baseline, stock + *amount),
                },
            );
            // Calculate consumption since anchor_date
            if let Some(anchor_date) = anchor_date {
                let consumption = self.calculate_consumption(anchor_date, at);
                (stock - consumption).max(0.)
            } else {
                0.
            }
        } else {
            0.
        }
    }

    pub fn calculate_days_remaining(&self, at: &NaiveDate) -> u64 {
        let stock = self.calculate_stock(at);
        match self.schedule {
            Schedule::Daily { amount } => (stock / amount).floor() as u64,
            Schedule::Weekly {
                day_of_week,
                amount,
            } => {
                let doses_left = (stock / amount).floor() as i64;
                let start_wd = at.weekday().num_days_from_sunday() as i64;
                let offset = (day_of_week as i64 - start_wd + 7) % 7;
                let offset = if offset == 0 { 7 } else { offset };
                let first_occurrence = *at + TimeDelta::days(offset);
                let nth_occurrence = first_occurrence + TimeDelta::days((doses_left - 1) * 7);
                (nth_occurrence - *at).num_days() as u64
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_daily_stock() {
        let medication = Medication {
            id: Uuid::nil(),
            name: String::from("ASS"),
            unit: String::from("Tabletten"),
            schedule: Schedule::Daily { amount: 5. },
            warning_threshold: 14,
            logs: Some(vec![LogEntry::Baseline {
                amount: 100.,
                date: date("2026-04-01"),
                note: None,
            }]),
        };

        assert_eq!(medication.calculate_stock(&date("2026-04-01")), 100.);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-01")), 20);
        assert_eq!(medication.calculate_stock(&date("2026-04-02")), 95.);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-02")), 19);
        assert_eq!(medication.calculate_stock(&date("2030-04-02")), 0.);
    }

    #[test]
    fn test_daily_stock_with_refill() {
        let medication = Medication {
            id: Uuid::nil(),
            name: String::from("ASS"),
            unit: String::from("Tabletten"),
            schedule: Schedule::Daily { amount: 1. },
            warning_threshold: 14,
            logs: Some(vec![
                LogEntry::Baseline {
                    amount: 100.,
                    date: date("2026-04-01"),
                    note: None,
                },
                LogEntry::Refill {
                    amount: 20.,
                    date: date("2026-04-05"),
                    note: None,
                },
            ]),
        };

        assert_eq!(medication.calculate_stock(&date("2026-04-01")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-04")), 97.);
        assert_eq!(medication.calculate_stock(&date("2026-04-05")), 116.);
        assert_eq!(medication.calculate_stock(&date("2026-04-06")), 115.);
    }

    #[test]
    fn test_weekly_stock() {
        let medication = Medication {
            id: Uuid::nil(),
            name: String::from("ASS"),
            unit: String::from("Tabletten"),
            schedule: Schedule::Weekly {
                day_of_week: 1,
                amount: 1.,
            },
            warning_threshold: 14,
            logs: Some(vec![LogEntry::Baseline {
                amount: 100.,
                date: date("2026-04-01"),
                note: None,
            }]),
        };

        assert_eq!(medication.calculate_stock(&date("2026-04-01")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-02")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-03")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-04")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-05")), 100.);
        assert_eq!(medication.calculate_stock(&date("2026-04-06")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-07")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-08")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-09")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-10")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-11")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-12")), 99.);
        assert_eq!(medication.calculate_stock(&date("2026-04-13")), 98.);
        assert_eq!(medication.calculate_stock(&date("2026-04-14")), 98.);
        assert_eq!(medication.calculate_stock(&date("2026-04-15")), 98.);
        assert_eq!(medication.calculate_stock(&date("2026-04-16")), 98.);
    }

    #[test]
    fn test_weekly_remaining_days() {
        let medication = Medication {
            id: Uuid::nil(),
            name: String::from("ASS"),
            unit: String::from("Tabletten"),
            schedule: Schedule::Weekly {
                day_of_week: 1,
                amount: 1.,
            },
            warning_threshold: 14,
            logs: Some(vec![LogEntry::Baseline {
                amount: 2.,
                date: date("2026-04-01"),
                note: None,
            }]),
        };

        assert_eq!(medication.calculate_days_remaining(&date("2026-04-01")), 12);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-02")), 11);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-03")), 10);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-04")), 9);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-05")), 8);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-06")), 7);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-07")), 6);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-08")), 5);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-09")), 4);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-10")), 3);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-11")), 2);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-12")), 1);
        assert_eq!(medication.calculate_days_remaining(&date("2026-04-13")), 0);
    }
}
