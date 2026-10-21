use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A fixed-time thing the human must attend, like a meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commitment {
    pub id: i64,
    pub title: String,
    pub area: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub location: Option<String>,
}
