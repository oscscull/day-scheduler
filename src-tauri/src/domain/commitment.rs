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
    /// The video call link, e.g. Google Meet or Zoom, for the alert's Join button.
    pub join_url: Option<String>,
    /// Where it came from, e.g. `manual` or `calendar:work`. Synced entries keep user overrides.
    pub source: String,
    /// The connector's own id for it, so a re-sync updates instead of duplicating.
    pub external_id: Option<String>,
    /// Hidden by the user or an agent; a re-sync keeps it hidden.
    pub hidden: bool,
}

/// A commitment as entered by hand or by an agent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitmentInput {
    pub title: String,
    pub area: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub location: Option<String>,
    #[serde(default)]
    pub join_url: Option<String>,
}
