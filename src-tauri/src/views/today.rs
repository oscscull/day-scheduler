use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;

use crate::domain::activity::Activity;
use crate::domain::commitment::Commitment;
use crate::domain::item::Stage;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::{activity, asks, commitments, items, runs, urgent};
use crate::today::{Priority, priorities, upcoming};

const ACTIVITY_SHOWN: u32 = 6;

/// Everything the Today screen shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub urgent: Vec<Urgent>,
    pub upcoming: Vec<Commitment>,
    pub priorities: Vec<Priority>,
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
            .take(2)
            .collect(),
        priorities: priorities(&ranked, &asks::list(conn)?, priority_limit),
        activity: activity::recent(conn, ACTIVITY_SHOWN)?,
        running: runs::live(conn)?.len(),
        in_flight,
    })
}
