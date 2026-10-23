//! Passing an item's turn. It's the only way a turn changes, so every change is in the item's
//! history with its reason.

pub mod due;
pub mod history;
mod pass;
pub mod work;

pub use pass::Pass;

use chrono::{DateTime, TimeDelta, Utc};

use crate::config::turns::TurnSettings;
use crate::domain::event::Change;
use crate::domain::life::{Active, Life, Phase, Whose};
use crate::domain::turn::Turn;
use crate::error::{AppError, AppResult};
use crate::store::{Db, events, items};

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

    /// When `turn` runs out, starting now. Fails on a length too large to add to the date.
    pub fn deadline(self, turn: Turn) -> AppResult<DateTime<Utc>> {
        let hours = match turn {
            Turn::You => self.limits.you_remind_hours,
            Turn::Agent => self.limits.agent_hours,
            Turn::Others => self.limits.others_hours,
            Turn::Later => self.limits.later_hours,
        };
        TimeDelta::try_hours(hours)
            .and_then(|d| self.now.checked_add_signed(d))
            .ok_or_else(|| AppError::BadInput(format!("[turns] {hours} hours is too long")))
    }
}

/// The deadline after a pass. An agent handing the turn to itself keeps its clock, so it can't
/// keep itself going forever.
fn until_after(old: Option<&Active>, pass: &Pass, clock: Clock<'_>) -> AppResult<DateTime<Utc>> {
    let kept = old
        .filter(|a| pass.to == Turn::Agent && a.whose == Whose::Agent)
        .and_then(|a| a.until);
    pass.until
        .or(kept)
        .map_or_else(|| clock.deadline(pass.to), Ok)
}

/// Hands the turn on. Unstarted work starts; done work has no turn, so nothing changes.
pub async fn pass(db: &Db, item_id: &str, pass: Pass, clock: Clock<'_>) -> AppResult<()> {
    let mut item = items::get(db, item_id).await?;
    let (phase, old) = match &item.life {
        Life::Done => return Ok(()),
        Life::Unstarted => (Phase::InProgress, None),
        Life::Active(a) => (a.phase, Some(a)),
    };
    let until = Some(until_after(old, &pass, clock)?);
    let started = old.is_none();
    item.life = Life::active_as(phase, pass.whose(), until);
    if started {
        item.state_since = clock.now;
    }
    items::save(db, &item).await?;
    if started {
        let stage = Change::Stage {
            stage: phase.stage(),
        };
        events::record(db, item_id, clock.now, &stage).await?;
    }
    let change = Change::Turn {
        turn: pass.to,
        why: pass.why,
    };
    events::record(db, item_id, clock.now, &change).await
}

/// Moves the clock without passing the turn: an item just in flight, or a reminder sent.
pub async fn reset_clock(db: &Db, item_id: &str, until: DateTime<Utc>) -> AppResult<()> {
    let mut item = items::get(db, item_id).await?;
    if let Life::Active(a) = &mut item.life {
        a.until = Some(until);
        items::save(db, &item).await?;
    }
    Ok(())
}
