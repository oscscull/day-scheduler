use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub summary: Option<String>,
}
