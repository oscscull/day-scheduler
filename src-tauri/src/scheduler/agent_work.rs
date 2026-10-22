//! What the agent is doing on an item: running, about to get an answer, working in a session
//! outside Quarterdeck, or nothing.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

use crate::config::settings::Settings;
use crate::domain::event::Change;
use crate::domain::item::WorkItem;
use crate::domain::session::SessionRef;
use crate::domain::turn::Turn;
use crate::error::{AppError, AppResult};
use crate::store::{Db, asks, events, runs, watch};
use crate::turns::due::Work;

/// What Quarterdeck knows of the agent's work across items, read once per tick.
pub struct Known {
    running: HashSet<String>,
    pending: HashSet<String>,
    sessions: HashMap<String, SessionRef>,
}

pub async fn known(db: &Db) -> AppResult<Known> {
    Ok(Known {
        running: runs::live(db)
            .await?
            .into_iter()
            .map(|r| r.target)
            .collect(),
        pending: asks::answered(db)
            .await?
            .into_iter()
            .map(|a| a.ask.item_id)
            .collect(),
        sessions: watch::all(db).await?,
    })
}

/// When anything other than a turn change was last heard about the item.
async fn last_heard(db: &Db, item_id: &str) -> AppResult<Option<DateTime<Utc>>> {
    let history = events::for_item(db, item_id).await?;
    Ok(history
        .iter()
        .find(|e| !matches!(e.change, Change::Turn { .. }))
        .map(|e| e.at))
}

/// A session outside Quarterdeck may be working on the item: it's idle once nothing has been heard
/// since Quarterdeck's own last run ended, or nothing for `quiet` minutes.
async fn idle(db: &Db, item: &WorkItem, known: &Known, quiet: Duration) -> AppResult<bool> {
    if !known.sessions.contains_key(&item.id) {
        return Ok(true);
    }
    let Some(heard) = last_heard(db, &item.id).await? else {
        return Ok(true);
    };
    let ran = runs::last_finished_for_item(db, &item.id).await?;
    Ok(ran.is_some_and(|r| r >= heard) || Utc::now() - heard >= quiet)
}

pub async fn work(db: &Db, item: &WorkItem, known: &Known, settings: &Settings) -> AppResult<Work> {
    if known.running.contains(&item.id) {
        return Ok(Work::Running);
    }
    if known.pending.contains(&item.id) {
        return Ok(Work::Pending);
    }
    let minutes = settings.turns.quiet_minutes;
    let quiet = Duration::try_minutes(minutes).ok_or_else(|| {
        AppError::BadInput(format!("[turns] quiet_minutes {minutes} is too long"))
    })?;
    if item.turn == Turn::Agent && !idle(db, item, known, quiet).await? {
        return Ok(Work::Elsewhere);
    }
    Ok(Work::Idle)
}
