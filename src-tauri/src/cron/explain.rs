use std::str::FromStr as _;

use chrono::{DateTime, Local, Utc};
use croner::Cron;
use serde::Serialize;

use super::fields::{CronField, fields};
use crate::error::AppResult;

/// Everything the UI shows about when something runs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    pub when: String,
    pub fields: Vec<CronField>,
    pub next_runs: Vec<DateTime<Utc>>,
}

/// Explains a cron expression and lists its next `count` run times after `now`.
pub fn explain(expr: &str, now: DateTime<Local>, count: usize) -> AppResult<Explanation> {
    let cron = Cron::from_str(expr)?;
    let next_runs = cron
        .iter_after(now)
        .take(count)
        .map(|t| t.with_timezone(&Utc))
        .collect();
    Ok(Explanation {
        when: cron.describe(),
        fields: fields(expr),
        next_runs,
    })
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    #[test]
    fn lists_next_runs_in_order() {
        let now = Local.with_ymd_and_hms(2026, 10, 21, 14, 32, 0).unwrap();
        let out = explain("0 9-18 * * 1-5", now, 3).unwrap();
        let hours: Vec<String> = out
            .next_runs
            .iter()
            .map(|t| t.with_timezone(&Local).format("%H:%M").to_string())
            .collect();
        assert_eq!(hours, ["15:00", "16:00", "17:00"]);
        assert!(!out.when.is_empty());
    }

    #[test]
    fn rejects_garbage() {
        assert!(explain("every tuesday", Local::now(), 1).is_err());
    }
}
