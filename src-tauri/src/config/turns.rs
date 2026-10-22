//! The `[turns]` section of `quarterdeck.toml`: how long each turn lasts before something happens.

use serde::{Deserialize, Serialize};

use crate::domain::turn::Turn;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TurnSettings {
    /// The agent's turn, counted from when it got it. Handing the turn to itself doesn't extend it.
    pub agent_hours: i64,
    /// Between reminders while it's the user's turn.
    pub you_remind_hours: i64,
    /// Waiting on reviewers, CI or another team before the turn comes back.
    pub others_hours: i64,
    /// Put off with no time given.
    pub later_hours: i64,
    /// Who gets the turn when others' wait runs out, unless the item says otherwise.
    pub others_then: Turn,
    /// How long an agent session working outside Quarterdeck can go quiet before it's prompted.
    pub quiet_minutes: i64,
}

impl Default for TurnSettings {
    fn default() -> Self {
        Self {
            agent_hours: 4,
            you_remind_hours: 24,
            others_hours: 24,
            later_hours: 72,
            others_then: Turn::You,
            quiet_minutes: 45,
        }
    }
}
