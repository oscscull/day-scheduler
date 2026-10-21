//! Calendars: private iCal links become meetings and appointments.

mod apply;
mod fetch;
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
        match fetch::ics(&feed.url).await {
            Ok(text) => {
                let mut occurrences = parse::occurrences(&text, from, to);
                occurrences.retain(|o| !settings.skips(feed, &o.title, o.all_day));
                app.db
                    .with(|conn| apply::apply(conn, feed, &occurrences, from))?;
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
