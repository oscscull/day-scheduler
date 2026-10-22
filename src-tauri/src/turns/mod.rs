//! Passing an item's turn. It's the only way a turn changes, so every change is in the item's
//! history with its reason.

pub mod due;

use chrono::{DateTime, Duration, Utc};

use crate::config::turns::TurnSettings;
use crate::domain::event::Change;
use crate::domain::item::WorkItem;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{Db, events, items};

/// Who the turn goes to, and why.
#[derive(Debug, Clone)]
pub struct Pass {
    pub to: Turn,
    /// What was done, or what ran out. Kept in the item's history.
    pub why: String,
    /// For others' turn: who it waits on.
    pub waiting_on: Option<String>,
    /// When the turn runs out; the configured length when unset.
    pub until: Option<DateTime<Utc>>,
    /// For others' or a later turn: who gets it after.
    pub then: Option<Turn>,
}

impl Pass {
    pub fn to(to: Turn, why: impl Into<String>) -> Self {
        Self {
            to,
            why: why.into(),
            waiting_on: None,
            until: None,
            then: None,
        }
    }

    pub fn waiting_on(self, who: impl Into<String>) -> Self {
        Self {
            waiting_on: Some(who.into()),
            ..self
        }
    }
}

/// The turn limits, and when a change happens.
#[derive(Debug, Clone, Copy)]
pub struct Clock<'a> {
    pub limits: &'a TurnSettings,
    pub now: DateTime<Utc>,
}

impl<'a> Clock<'a> {
    pub fn now(limits: &'a TurnSettings) -> Self {
        Self {
            limits,
            now: Utc::now(),
        }
    }

    pub fn deadline(self, turn: Turn) -> DateTime<Utc> {
        deadline(turn, self.limits, self.now)
    }
}

/// How long `turn` lasts from `now`.
pub fn deadline(turn: Turn, limits: &TurnSettings, now: DateTime<Utc>) -> DateTime<Utc> {
    let hours = match turn {
        Turn::You => limits.you_remind_hours,
        Turn::Agent => limits.agent_hours,
        Turn::Others => limits.others_hours,
        Turn::Later => limits.later_hours,
    };
    now + Duration::hours(hours)
}

/// The deadline after a pass. An agent handing the turn to itself keeps its clock, so it can't
/// keep itself going forever.
fn until_after(item: &WorkItem, pass: &Pass, clock: Clock<'_>) -> DateTime<Utc> {
    let kept = (pass.to == Turn::Agent && item.turn == Turn::Agent)
        .then_some(item.turn_until)
        .flatten();
    pass.until
        .or(kept)
        .unwrap_or_else(|| clock.deadline(pass.to))
}

pub async fn pass(db: &Db, item_id: &str, pass: Pass, clock: Clock<'_>) -> AppResult<()> {
    let mut item = items::get(db, item_id).await?;
    item.turn_until = Some(until_after(&item, &pass, clock));
    item.turn = pass.to;
    item.waiting_on = pass.waiting_on.filter(|_| pass.to == Turn::Others);
    item.then_turn = pass
        .then
        .filter(|_| matches!(pass.to, Turn::Others | Turn::Later));
    items::save(db, &item).await?;
    let change = Change::Turn {
        turn: pass.to,
        why: pass.why,
    };
    events::record(db, item_id, clock.now, &change).await
}

/// Moves the clock without passing the turn: an item just in flight, or a reminder sent.
pub async fn reset_clock(db: &Db, item_id: &str, until: DateTime<Utc>) -> AppResult<()> {
    let mut item = items::get(db, item_id).await?;
    item.turn_until = Some(until);
    items::save(db, &item).await
}

/// Why the item last changed hands, if it ever did.
pub async fn last_why(db: &Db, item_id: &str) -> AppResult<Option<String>> {
    let history = events::for_item(db, item_id).await?;
    Ok(history.into_iter().find_map(|e| match e.change {
        Change::Turn { why, .. } => Some(why),
        Change::Created
        | Change::Stage { .. }
        | Change::Status { .. }
        | Change::Label { .. }
        | Change::Note { .. } => None,
    }))
}

/// Whether the agent handed the turn on since `since`, through `pass` (its own, or anyone's).
pub async fn passed_since(db: &Db, item_id: &str, since: DateTime<Utc>) -> AppResult<bool> {
    let history = events::for_item(db, item_id).await?;
    Ok(history
        .iter()
        .any(|e| e.at > since && matches!(e.change, Change::Turn { .. })))
}
