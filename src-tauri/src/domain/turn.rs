//! Whose move an item is. Every item in flight has a turn and a deadline, so nothing idles.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Turn {
    /// The user acts next; they're reminded when the deadline passes.
    #[default]
    You,
    /// The agent acts next; Quarterdeck prompts it until it hands the turn on.
    Agent,
    /// Reviewers, CI or another team; the turn comes back when the wait runs out.
    Others,
    /// Put off until the deadline, then back to whoever is next.
    Later,
}

impl Turn {
    /// The name it's stored and sent under, e.g. in `quarterdeck.toml`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::You => "you",
            Self::Agent => "agent",
            Self::Others => "others",
            Self::Later => "later",
        }
    }
}
