//! Runs connector syncs on their own interval, in the background.

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::settings::Settings;
use crate::connectors::{github, status};
use crate::error::AppResult;
use crate::store::meta;

const GITHUB_SYNC: &str = "connector.github.last_sync";

/// Starts a GitHub sync if one is due. Its outcome is recorded for the Agent tab.
pub fn sync_if_due(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    if !settings.github.enabled {
        return Ok(());
    }
    let app = handle.state::<App>();
    let now = Utc::now();
    let last = app.db.with(|c| meta::time(c, GITHUB_SYNC))?;
    if last.is_some_and(|t| now - t < Duration::minutes(settings.github.poll_minutes)) {
        return Ok(());
    }
    // Claim this slot now so the next tick doesn't start a second sync.
    app.db.with(|c| meta::set_time(c, GITHUB_SYNC, now))?;
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let result = github::sync(&handle).await;
        let state = handle.state::<App>();
        let error = result.as_ref().err().map(ToString::to_string);
        drop(
            state
                .db
                .with(|c| status::record(c, github::SOURCE, error.as_deref())),
        );
        if let Ok(fired) = result {
            drop(super::events::start_for(&handle, &fired));
        }
    });
    Ok(())
}
