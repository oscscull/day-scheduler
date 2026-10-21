//! A schedule must live on the machine that runs its area, or it never fires.

use serde_json::{Value, json};

use super::client;
use crate::config::settings::Settings;
use crate::domain::schedule::{Schedule, Trigger};
use crate::error::AppResult;

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

/// [`create_elsewhere`] off the calling thread, since it waits on the network.
pub async fn create_elsewhere_async(
    settings: Settings,
    schedule: Schedule,
) -> AppResult<Option<String>> {
    tauri::async_runtime::spawn_blocking(move || create_elsewhere(&settings, &schedule))
        .await
        .map_err(|e| crate::error::AppError::Desktop(e.to_string()))?
}
