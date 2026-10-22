use chrono::Local;
use serde::Serialize;

use crate::config::pause::Pause;
use crate::config::schedules::Loaded;
use crate::connectors::{calendar, github, status};
use crate::cron::explain::{Explanation, explain};
use crate::domain::run::Run;
use crate::domain::schedule::{Schedule, Trigger};
use crate::error::AppResult;
use crate::store::{Db, runs};
use crate::views::agent_turns::{self, AgentTurn};
use crate::views::live::{self, LiveRun};

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
    pub live: Vec<LiveRun>,
    /// Items that are the agent's turn, running or about to be.
    pub turns: Vec<AgentTurn>,
    pub schedules: Vec<ScheduleView>,
    pub problems: Vec<String>,
    pub pause: Option<Pause>,
    pub connections: Vec<status::Status>,
}

pub async fn build(db: &Db, loaded: Loaded, pause: Option<Pause>) -> AppResult<AgentView> {
    let mut problems = loaded.problems;
    let mut schedules = Vec::with_capacity(loaded.schedules.len());
    for schedule in loaded.schedules {
        let explanation = match &schedule.trigger {
            Trigger::Cron { expr } => explain(expr, Local::now(), NEXT_RUNS)
                .map_err(|e| problems.push(format!("{}: {e}", schedule.id)))
                .ok(),
            Trigger::Event { .. } => None,
        };
        let recent = runs::recent_for(db, &schedule.id, RECENT_RUNS).await?;
        schedules.push(ScheduleView {
            schedule,
            explanation,
            runs: recent,
        });
    }
    Ok(AgentView {
        live: live::build(db).await?,
        turns: agent_turns::build(db).await?,
        schedules,
        problems,
        pause,
        connections: vec![
            status::read(db, github::SOURCE).await?,
            status::read(db, calendar::SOURCE).await?,
        ],
    })
}
