use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::item::{Stage, Status};
use super::turn::Turn;

/// One change in an item's history. Stored as data, not sentences, so the UI can word it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Change {
    Created,
    Stage {
        stage: Stage,
    },
    Status {
        status: Option<Status>,
    },
    Label {
        label: String,
    },
    /// A free-text note, e.g. from an agent: "opened PR #821".
    Note {
        text: String,
    },
    /// The turn passed, and why: what was done, or what ran out.
    Turn {
        turn: Turn,
        why: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemEvent {
    pub id: i64,
    pub item_id: String,
    pub at: DateTime<Utc>,
    pub change: Change,
}
