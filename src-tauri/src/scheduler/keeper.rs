//! Keeps every item in flight moving: each tick, an item whose clock calls for something gets it.
//! The agent is prompted on its turn, the user reminded on theirs, and a wait that runs out passes
//! the turn on. The rules are in `turns::due`; this carries them out.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Manager as _};

use super::agent_prompt;
use crate::app::App;
use crate::config::settings::Settings;
use crate::domain::event::Change;
use crate::domain::item::WorkItem;
use crate::domain::session::SessionRef;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::notify;
use crate::store::{Db, asks, events, items, runs, watch};
use crate::turns::due::{Due, Work, due};
use crate::turns::{self, Clock, Pass};

/// What Quarterdeck knows of the agent's work across items, read once per tick.
struct Known {
    running: HashSet<String>,
    pending: HashSet<String>,
    sessions: HashMap<String, SessionRef>,
}

async fn known(db: &Db) -> AppResult<Known> {
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

async fn work(db: &Db, item: &WorkItem, known: &Known, settings: &Settings) -> AppResult<Work> {
    if known.running.contains(&item.id) {
        return Ok(Work::Running);
    }
    if known.pending.contains(&item.id) {
        return Ok(Work::Pending);
    }
    let quiet = Duration::minutes(settings.turns.quiet_minutes);
    if item.turn == Turn::Agent && !idle(db, item, known, quiet).await? {
        return Ok(Work::Elsewhere);
    }
    Ok(Work::Idle)
}

async fn act(handle: &AppHandle, settings: &Settings, item: &WorkItem, due: Due) -> AppResult<()> {
    let db = &handle.state::<App>().db;
    let clock = Clock::now(&settings.turns);
    match due {
        Due::Wait => Ok(()),
        Due::StartClock => turns::reset_clock(db, &item.id, clock.deadline(item.turn)?).await,
        Due::Prompt => agent_prompt::carry_on(handle, settings, item).await,
        Due::Remind => {
            turns::reset_clock(db, &item.id, clock.deadline(Turn::You)?).await?;
            tell_user(handle, item, "Still your turn");
            Ok(())
        }
        Due::PassTo { to, why } => {
            turns::pass(db, &item.id, Pass::to(to, why), clock).await?;
            if to == Turn::You {
                tell_user(handle, item, "Your turn");
            }
            Ok(())
        }
    }
}

/// A notification is a courtesy; the turn is on Today either way.
fn tell_user(handle: &AppHandle, item: &WorkItem, heading: &str) {
    let title = format!("{heading}: {}", item.title);
    drop(notify::send(handle, &title, &item.state_label));
}

pub async fn check(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let known = known(&app.db).await?;
    let mine = items::list(&app.db).await?;
    for item in mine
        .iter()
        .filter(|i| settings.node_for(&i.area) == settings.node)
    {
        let work = work(&app.db, item, &known, settings).await?;
        let due = due(item, work, &settings.turns, Utc::now());
        act(handle, settings, item, due).await?;
    }
    Ok(())
}
