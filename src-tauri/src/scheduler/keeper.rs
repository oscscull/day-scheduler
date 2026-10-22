//! Keeps every item in flight moving: each tick, an item whose clock calls for something gets it.
//! The agent is prompted on its turn, the user reminded on theirs, and a wait that runs out passes
//! the turn on. The rules are in `turns::due`; this carries them out.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use super::agent_prompt;
use super::agent_work::{self, Known};
use crate::app::App;
use crate::config::settings::Settings;
use crate::domain::item::WorkItem;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::notify;
use crate::store::items;
use crate::turns::due::{Due, due};
use crate::turns::{self, Clock, Pass};

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

async fn keep(
    handle: &AppHandle,
    settings: &Settings,
    item: &WorkItem,
    known: &Known,
) -> AppResult<()> {
    let db = &handle.state::<App>().db;
    let work = agent_work::work(db, item, known, settings).await?;
    let due = due(item, work, &settings.turns, Utc::now());
    act(handle, settings, item, due).await
}

/// Every item gets its turn kept, even when one fails; the first failure is reported.
pub async fn check(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let known = agent_work::known(&app.db).await?;
    let mine = items::list(&app.db).await?;
    let mut first = Ok(());
    for item in mine
        .iter()
        .filter(|i| settings.node_for(&i.area) == settings.node)
    {
        let kept = keep(handle, settings, item, &known).await;
        if first.is_ok() {
            first = kept;
        }
    }
    first
}
