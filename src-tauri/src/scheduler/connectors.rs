//! Runs connector syncs on their own intervals, in the background.

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter as _, Manager as _};

use crate::app::App;
use crate::config::settings::Settings;
use crate::connectors::{calendar, github, status};
use crate::error::AppResult;
use crate::store::meta;

/// The screen reloads when it hears this, so a sync's changes show without waiting.
pub const SYNCED: &str = "synced";

/// True if `name` hasn't synced within `gap`, claiming the slot so the next tick doesn't start
/// a second sync.
pub(super) async fn claim(handle: &AppHandle, name: &str, gap: Duration) -> AppResult<bool> {
    let app = handle.state::<App>();
    let key = format!("connector.{name}.last_sync");
    let now = Utc::now();
    let last = meta::time(&app.db, &key).await?;
    if last.is_some_and(|t| now - t < gap) {
        return Ok(false);
    }
    meta::set_time(&app.db, &key, now).await?;
    Ok(true)
}

async fn record(handle: &AppHandle, name: &str, error: Option<&str>) {
    let state = handle.state::<App>();
    // Status is for display; failing to save it must not stop the sync loop.
    drop(status::record(&state.db, name, error).await);
}

/// Syncs GitHub in the background, starts any event schedules it fired, and tells the screen.
pub(super) fn spawn_github(handle: &AppHandle) {
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let result = github::sync(&handle).await;
        let error = result.as_ref().err().map(ToString::to_string);
        record(&handle, github::SOURCE, error.as_deref()).await;
        if let Ok(fired) = result {
            drop(super::events::start_for(&handle, &fired).await);
        }
        // The screen also reloads on its own timer, so a missed event only delays it.
        drop(handle.emit(SYNCED, ()));
    });
}

pub async fn sync_if_due(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let every = Duration::minutes(settings.github.poll_minutes);
    if settings.github.enabled && claim(handle, github::SOURCE, every).await? {
        spawn_github(handle);
    }
    if !settings.calendar.feeds.is_empty()
        && claim(
            handle,
            calendar::SOURCE,
            Duration::minutes(settings.calendar.poll_minutes),
        )
        .await?
    {
        let handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            let result = calendar::sync(&handle).await;
            let error = result.err().map(|e| e.to_string());
            record(&handle, calendar::SOURCE, error.as_deref()).await;
        });
    }
    Ok(())
}
