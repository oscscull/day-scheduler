//! Installs new releases from GitHub by itself. Every merge to `main` publishes one (see
//! `.github/workflows/release.yml`), so every machine runs the same build without rebuilding.
//! An update waits until it can't interrupt: no Claude run is live and the window isn't in use.

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _};
use tauri_plugin_updater::UpdaterExt as _;

use crate::app::App;
use crate::config::settings::Settings;
use crate::desktop;
use crate::error::{AppError, AppResult};
use crate::store::{meta, runs};

const LAST_CHECK: &str = "updates.last_check";

fn update_error(e: impl std::fmt::Display) -> AppError {
    AppError::Desktop(format!("couldn't update: {e}"))
}

/// Starts a check when the last one is `check_minutes` old.
pub fn check_if_due(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let now = Utc::now();
    let every = Duration::minutes(settings.updates.check_minutes);
    let last = app.db.with(|c| meta::time(c, LAST_CHECK))?;
    if !settings.updates.enabled || last.is_some_and(|t| now - t < every) {
        return Ok(());
    }
    app.db.with(|c| meta::set_time(c, LAST_CHECK, now))?;
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        // A failed check is retried at the next one; nothing else depends on it.
        drop(install_when_idle(&handle).await);
    });
    Ok(())
}

/// No Claude run would be cut off, and the user isn't looking at the window.
fn idle(handle: &AppHandle) -> AppResult<bool> {
    let running = !handle.state::<App>().db.with(runs::live)?.is_empty();
    let in_use = handle
        .get_webview_window(desktop::MAIN)
        .is_some_and(|w| w.is_focused().unwrap_or(false));
    Ok(!running && !in_use)
}

async fn install_when_idle(handle: &AppHandle) -> AppResult<()> {
    let updater = handle.updater().map_err(update_error)?;
    let Some(update) = updater.check().await.map_err(update_error)? else {
        return Ok(());
    };
    if !idle(handle)? {
        // Try again on the next tick rather than in `check_minutes`.
        return handle
            .state::<App>()
            .db
            .with(|c| meta::forget(c, LAST_CHECK));
    }
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(update_error)?;
    handle.restart()
}
