use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Where a piece of work is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    UpNext,
    InProgress,
    InReview,
    Done,
}

/// What a piece of work is waiting on, shown as a chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Decision,
    Review,
    Ci,
    Dependency,
    Proceeding,
    Queued,
}

/// One thing in play: a ticket, PR or personal task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub state_label: String,
    pub stage: Stage,
    pub status: Option<Status>,
    pub holder: String,
    pub area: String,
    pub opened_at: DateTime<Utc>,
    pub rank: i64,
}
