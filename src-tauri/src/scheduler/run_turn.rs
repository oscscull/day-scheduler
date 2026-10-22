//! A run about an item happens on the agent's turn, and ends with the turn handed on: by the agent
//! through `pass_turn`, or back to the user with what the run said.

use chrono::{DateTime, Utc};

use crate::config::turns::TurnSettings;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{Db, items};
use crate::turns::{self, Clock, Pass};

/// Gives the agent the turn, unless it has it already.
pub async fn take(db: &Db, (item_id, what): (&str, &str), limits: &TurnSettings) -> AppResult<()> {
    if items::get(db, item_id).await?.turn == Turn::Agent {
        return Ok(());
    }
    turns::pass(db, item_id, Pass::to(Turn::Agent, what), Clock::now(limits)).await
}

/// A run about an item that finished: when it started, and what it said last.
#[derive(Debug, Clone, Copy)]
pub struct Finished<'a> {
    pub item_id: &'a str,
    pub started: DateTime<Utc>,
    pub said: &'a str,
}

/// After a run that finished: if the agent didn't hand the turn on, it goes to the user.
pub async fn end(db: &Db, run: Finished<'_>, limits: &TurnSettings) -> AppResult<()> {
    let item_id = run.item_id;
    let still_agents = items::get(db, item_id).await?.turn == Turn::Agent;
    if !still_agents || turns::passed_since(db, item_id, run.started).await? {
        return Ok(());
    }
    let why = format!("the agent stopped without handing over: {}", run.said);
    turns::pass(db, item_id, Pass::to(Turn::You, why), Clock::now(limits)).await
}
