use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;

use crate::domain::activity::Activity;
use crate::domain::commitment::Commitment;
use crate::domain::item::Stage;
use crate::domain::todo::Todo;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::{activity, asks, commitments, items, runs, todos, urgent};
use crate::today::{Priority, priorities, upcoming};

const ACTIVITY_SHOWN: u32 = 12;
/// Enough upcoming commitments for any one area to still show its next two.
const UPCOMING_SHOWN: usize = 12;

/// Everything the Today screen shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub urgent: Vec<Urgent>,
    pub upcoming: Vec<Commitment>,
    /// Every ask in rank order; the UI shows the first `priority_limit` in the chosen area.
    pub priorities: Vec<Priority>,
    pub priority_limit: usize,
    pub todos: Vec<Todo>,
    pub activity: Vec<Activity>,
    pub running: usize,
    pub in_flight: usize,
}

pub fn build(conn: &Connection, priority_limit: usize, now: DateTime<Utc>) -> AppResult<TodayView> {
    let ranked = items::list(conn)?;
    let in_flight = ranked
        .iter()
        .filter(|i| !matches!(i.stage, Stage::UpNext | Stage::Done))
        .count();
    Ok(TodayView {
        urgent: urgent::list(conn)?,
        upcoming: upcoming(commitments::list(conn)?, now)
            .into_iter()
            .take(UPCOMING_SHOWN)
            .collect(),
        priorities: priorities(&ranked, &asks::list(conn)?, usize::MAX),
        priority_limit,
        todos: todos::list(conn)?,
        activity: activity::recent(conn, ACTIVITY_SHOWN)?,
        running: runs::live(conn)?.len(),
        in_flight,
    })
}
