use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A low-key note about something Claude did.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: i64,
    pub text: String,
    pub area: String,
    pub at: DateTime<Utc>,
    pub unread: bool,
}
