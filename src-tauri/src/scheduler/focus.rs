//! Coming back to the window syncs GitHub at once, so a PR merged in the browser leaves the screen
//! right away instead of at the next poll.

use chrono::Duration;
use tauri::{AppHandle, Manager as _};

use super::connectors::{claim, spawn_github};
use crate::app::App;
use crate::connectors::github;
use crate::error::AppResult;

pub fn sync_on_focus(handle: &AppHandle) -> AppResult<()> {
    let settings = handle.state::<App>().settings()?.github;
    let gap = Duration::seconds(settings.focus_sync_seconds);
    if settings.enabled && claim(handle, github::SOURCE, gap)? {
        spawn_github(handle);
    }
    Ok(())
}
