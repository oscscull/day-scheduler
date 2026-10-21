use serde::{Deserialize, Serialize};

/// Things app code can detect by itself, with no tokens spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    CiFailed,
    PrNewComments,
}

/// When a schedule fires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    Cron { expr: String },
    Event { kind: EventKind },
}

/// What a schedule may do when it wants to change something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Autonomy {
    Ask,
    Act,
}

/// What a schedule runs against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    EachPr,
    EachItem,
    One,
    Nothing,
}

/// A recurring job for Claude.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub trigger: Trigger,
    /// Which area this belongs to; the area decides the machine it runs on.
    pub area: String,
    pub scope: Scope,
    pub autonomy: Autonomy,
    pub enabled: bool,
    /// The plain-English request this schedule was made from, if any.
    pub request: String,
}
