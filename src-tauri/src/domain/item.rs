use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Where a piece of work is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    UpNext,
    InProgress,
    InReview,
    Done,
}

/// What a piece of work is waiting on, shown as a chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Decision,
    Review,
    /// CI is running.
    Ci,
    /// CI failed: an agent should be fixing it.
    CiFailed,
    Dependency,
    /// Approved with CI passing: the user can merge it.
    Approved,
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
    /// Where the work lives: a ticket, PR or doc.
    pub link: Option<String>,
    /// `manual`, or the connector that made it, e.g. `github`.
    pub source: String,
    /// The connector's own id, e.g. `owner/repo#12`.
    pub external_id: Option<String>,
    pub opened_at: DateTime<Utc>,
    /// When the stage, status or label last changed.
    pub state_since: DateTime<Utc>,
    pub rank: i64,
}

/// The editable parts of an item, as sent from the UI or an agent. An empty `id` creates one.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    pub id: String,
    pub title: String,
    pub state_label: String,
    pub stage: Stage,
    pub status: Option<Status>,
    pub holder: String,
    pub area: String,
    pub link: Option<String>,
}
