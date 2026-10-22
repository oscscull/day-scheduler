//! The items that are the agent's turn, so the Agent tab and In flight show the same work.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::item::{Stage, WorkItem};
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{Db, items, runs, watch};

/// Where the agent's turn on an item stands.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Doing {
    /// A Quarterdeck run is working on it.
    Running,
    /// A session outside Quarterdeck holds it.
    Session,
    /// Quarterdeck prompts it on the next tick.
    Starting,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTurn {
    pub item_id: String,
    pub title: String,
    pub area: String,
    pub doing: Doing,
    /// When the agent's turn runs out and passes back to the user.
    pub until: Option<DateTime<Utc>>,
}

const fn agents(item: &WorkItem) -> bool {
    matches!(item.turn, Turn::Agent) && matches!(item.stage, Stage::InProgress | Stage::InReview)
}

pub async fn build(db: &Db) -> AppResult<Vec<AgentTurn>> {
    let running: Vec<String> = runs::live(db)
        .await?
        .into_iter()
        .map(|r| r.target)
        .collect();
    let sessions = watch::all(db).await?;
    Ok(items::list(db)
        .await?
        .into_iter()
        .filter(agents)
        .map(|i| {
            let doing = if running.contains(&i.id) {
                Doing::Running
            } else if sessions.contains_key(&i.id) {
                Doing::Session
            } else {
                Doing::Starting
            };
            AgentTurn {
                item_id: i.id,
                title: i.title,
                area: i.area,
                doing,
                until: i.turn_until,
            }
        })
        .collect())
}
