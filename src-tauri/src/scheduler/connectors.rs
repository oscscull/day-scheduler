//! Runs connector syncs on their own intervals, in the background.

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::settings::Settings;
use crate::connectors::{calendar, github, status};
use crate::error::AppResult;
use crate::store::meta;

/// True if `name` hasn't synced for `minutes`, claiming the slot so the next tick doesn't start
/// a second sync.
fn claim(handle: &AppHandle, name: &str, minutes: i64) -> AppResult<bool> {
    let app = handle.state::<App>();
    let key = format!("connector.{name}.last_sync");
    let now = Utc::now();
    let last = app.db.with(|c| meta::time(c, &key))?;
    if last.is_some_and(|t| now - t < Duration::minutes(minutes)) {
        return Ok(false);
    }
    app.db.with(|c| meta::set_time(c, &key, now))?;
    Ok(true)
}

fn record(handle: &AppHandle, name: &str, error: Option<&str>) {
    let state = handle.state::<App>();
    // Status is for display; failing to save it must not stop the sync loop.
    drop(state.db.with(|c| status::record(c, name, error)));
}

pub fn sync_if_due(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    if settings.github.enabled && claim(handle, github::SOURCE, settings.github.poll_minutes)? {
        let handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            let result = github::sync(&handle).await;
            let error = result.as_ref().err().map(ToString::to_string);
            record(&handle, github::SOURCE, error.as_deref());
            if let Ok(fired) = result {
                drop(super::events::start_for(&handle, &fired));
            }
        });
    }
    if !settings.calendar.feeds.is_empty()
        && claim(handle, calendar::SOURCE, settings.calendar.poll_minutes)?
    {
        let handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            let result = calendar::sync(&handle).await;
            let error = result.err().map(|e| e.to_string());
            record(&handle, calendar::SOURCE, error.as_deref());
        });
    }
    Ok(())
}
