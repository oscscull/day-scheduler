use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How a finished run ended. Live runs have no outcome yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ok,
    Failed,
    /// Never started, e.g. the machine was asleep or offline when it was due.
    Missed,
}

/// One piece of work Claude did or is doing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: i64,
    pub schedule_id: Option<String>,
    pub what: String,
    pub target: String,
    pub node: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub outcome: Option<Outcome>,
    pub summary: Option<String>,
}
