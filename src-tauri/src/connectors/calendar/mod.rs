//! Calendars: private iCal links and macOS Calendar app calendars become meetings and appointments.

mod apply;
mod fetch;
mod links;
pub mod macos;
pub mod parse;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::error::{AppError, AppResult};

pub const SOURCE: &str = "calendar";

/// Syncs every configured feed. One broken feed doesn't stop the others; their errors are
/// reported together.
pub async fn sync(handle: &AppHandle) -> AppResult<()> {
    let app = handle.state::<App>();
    let all = app.settings()?;
    let settings = all.calendar.clone();
    let now = Utc::now();
    let (from, to) = (
        now - Duration::days(1),
        now + Duration::days(settings.days_ahead),
    );
    let mut errors = Vec::new();
    // Another machine syncs the calendars of the areas it runs.
    for feed in settings
        .feeds
        .iter()
        .filter(|f| all.node_for(&f.area) == all.node)
    {
        // A feed that can't be fetched or read is an error, never an empty calendar.
        match fetch::occurrences(feed, from, to).await {
            Ok(mut occurrences) => {
                occurrences.retain(|o| !settings.skips(feed, &o.title, o.all_day));
                apply::apply(&app.db, feed, &occurrences, from).await?;
            }
            Err(e) => errors.push(format!("{}: {e}", feed.name)),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Connector(errors.join("; ")))
    }
}
