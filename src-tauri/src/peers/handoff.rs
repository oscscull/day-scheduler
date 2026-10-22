//! A schedule must live on the machine that runs its area, or it never fires. New schedules are
//! created there; one whose area now belongs elsewhere (edited, or the area moved) is handed over:
//! created there, then deleted here. Only once both machines agree on the areas, or two machines
//! that disagree would pass a schedule back and forth.

use serde_json::{Value, json};

use super::{areas, client};
use crate::config::settings::Settings;
use crate::config::sync::Peer;
use crate::config::{Config, schedules};
use crate::domain::schedule::{Schedule, Trigger};
use crate::error::{AppError, AppResult};
use crate::store::Db;

/// The `create_schedule` arguments for a schedule.
fn args(s: &Schedule) -> Value {
    let (cron, event) = match &s.trigger {
        Trigger::Cron { expr } => (Some(expr.clone()), None),
        Trigger::Event { kind } => (None, Some(*kind)),
    };
    json!({
        "name": s.name, "area": s.area, "instructions": s.prompt, "cron": cron, "event": event,
        "scope": s.scope, "autonomy": s.autonomy, "allowed_tools": s.allowed_tools, "request": s.request,
    })
}

/// Creates the schedule on the peer that runs its area, if that's not this machine. Returns the
/// peer's id for it, or `None` when it belongs here.
pub fn create_elsewhere(settings: &Settings, schedule: &Schedule) -> AppResult<Option<String>> {
    let owner = settings.node_for(&schedule.area);
    let Some(peer) = settings.sync.peer(owner).filter(|_| owner != settings.node) else {
        return Ok(None);
    };
    let id = client::call(peer, "create_schedule", &args(schedule))?;
    Ok(Some(id.as_str().unwrap_or_default().to_owned()))
}

/// Runs blocking network work off the calling thread.
async fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| AppError::Desktop(e.to_string()))?
}

/// [`create_elsewhere`] off the calling thread, since it waits on the network.
pub async fn create_elsewhere_async(
    settings: Settings,
    schedule: Schedule,
) -> AppResult<Option<String>> {
    off_thread(move || create_elsewhere(&settings, &schedule)).await
}

/// The machine that should hold `schedule`, when that isn't this one.
fn owner<'a>(settings: &'a Settings, schedule: &Schedule) -> Option<&'a str> {
    Some(settings.node_for(&schedule.area)).filter(|o| *o != settings.node)
}

/// The owner's peer, if the schedule can move there now: known, and agreeing on areas.
async fn ready(db: &Db, settings: &Settings, owner: &str) -> AppResult<Option<Peer>> {
    let Some(peer) = settings.sync.peer(owner) else {
        return Ok(None);
    };
    Ok(areas::agreed(db, settings, owner)
        .await?
        .then(|| peer.clone()))
}

/// Creates the schedule on `peer`, on or off as it was, then deletes it here. Returns its new id.
fn hand_over(config: &Config, peer: &Peer, schedule: &Schedule) -> AppResult<String> {
    let id = client::call(peer, "create_schedule", &args(schedule))?;
    let id = id.as_str().unwrap_or_default().to_owned();
    if !schedule.enabled {
        client::call(
            peer,
            "set_schedule_enabled",
            &json!({ "id": id, "enabled": false }),
        )?;
    }
    schedules::delete(config, &schedule.id)?;
    Ok(id)
}

/// Hands every schedule held here but owned elsewhere to its owner. One that can't move yet stays
/// and is tried again next time.
pub async fn move_misplaced(db: &Db, config: &Config, settings: &Settings) -> AppResult<()> {
    for schedule in schedules::list(config)?.schedules {
        let Some(owner) = owner(settings, &schedule) else {
            continue;
        };
        if let Some(peer) = ready(db, settings, owner).await? {
            let config = config.clone();
            drop(off_thread(move || hand_over(&config, &peer, &schedule)).await);
        }
    }
    Ok(())
}

/// Runs a schedule on the machine that owns it, moving it there first. `false` when it runs here.
pub async fn run_elsewhere(
    db: &Db,
    config: &Config,
    settings: &Settings,
    schedule: Schedule,
) -> AppResult<bool> {
    let Some(owner) = owner(settings, &schedule) else {
        return Ok(false);
    };
    let Some(peer) = ready(db, settings, owner).await? else {
        return Err(AppError::BadInput(format!(
            "{} runs on {owner}, which isn't reachable or doesn't have the same areas yet",
            schedule.name
        )));
    };
    let config = config.clone();
    off_thread(move || {
        let id = hand_over(&config, &peer, &schedule)?;
        client::call(&peer, "run_schedule", &json!({ "id": id }))
    })
    .await?;
    Ok(true)
}
