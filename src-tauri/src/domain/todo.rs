use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Something only the user can do: an errand, a booking, "make dinner on Friday".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub area: String,
    pub due: Option<NaiveDate>,
    pub done_at: Option<DateTime<Utc>>,
}
