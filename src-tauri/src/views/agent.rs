use chrono::Local;
use rusqlite::Connection;
use serde::Serialize;

use crate::config::schedules::Loaded;
use crate::cron::explain::{Explanation, explain};
use crate::domain::run::Run;
use crate::domain::schedule::{Schedule, Trigger};
use crate::error::AppResult;
use crate::store::runs;

const NEXT_RUNS: usize = 3;
const RECENT_RUNS: u32 = 3;

/// A schedule with when it runs and what it did lately.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleView {
    #[serde(flatten)]
    pub schedule: Schedule,
    /// Missing for event triggers, which have no clock time.
    pub explanation: Option<Explanation>,
    pub runs: Vec<Run>,
}

/// Everything the Agent screen shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    pub live: Vec<Run>,
    pub schedules: Vec<ScheduleView>,
    pub problems: Vec<String>,
}

pub fn build(conn: &Connection, loaded: Loaded) -> AppResult<AgentView> {
    let mut problems = loaded.problems;
    let mut schedules = Vec::with_capacity(loaded.schedules.len());
    for schedule in loaded.schedules {
        let explanation = match &schedule.trigger {
            Trigger::Cron { expr } => explain(expr, Local::now(), NEXT_RUNS)
                .map_err(|e| problems.push(format!("{}: {e}", schedule.id)))
                .ok(),
            Trigger::Event { .. } => None,
        };
        let recent = runs::recent_for(conn, &schedule.id, RECENT_RUNS)?;
        schedules.push(ScheduleView {
            schedule,
            explanation,
            runs: recent,
        });
    }
    Ok(AgentView {
        live: runs::live(conn)?,
        schedules,
        problems,
    })
}
