//! Calendar feeds from the Agent tab. Links are secret, so they go into `quarterdeck.toml` but never
//! back out to the screen.

use serde::Serialize;
use tauri::State;

use crate::app::App;
use crate::config::calendar::Feed;
use crate::config::edit;
use crate::connectors::{calendar, status};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct CalendarView {
    name: String,
    area: String,
}

#[tauri::command]
pub fn calendars(app: State<'_, App>) -> AppResult<Vec<CalendarView>> {
    let feeds = app.settings()?.calendar.feeds;
    Ok(feeds
        .into_iter()
        .map(|f| CalendarView {
            name: f.name,
            area: f.area,
        })
        .collect())
}

/// `webcal://` is the same feed over https.
fn https(url: &str) -> AppResult<String> {
    let url = url.trim();
    if let Some(rest) = url.strip_prefix("webcal://") {
        return Ok(format!("https://{rest}"));
    }
    if url.starts_with("https://") || url.starts_with("http://") {
        return Ok(url.to_owned());
    }
    Err(AppError::BadInput(
        "a calendar link starts with https:// or webcal://".to_owned(),
    ))
}

/// A name not already taken, based on the area's name: "Work", then "Work 2".
fn free_name(taken: &[Feed], base: &str) -> String {
    let free = |n: &String| !taken.iter().any(|f| &f.name == n);
    (1..=taken.len() + 1)
        .map(|i| {
            if i == 1 {
                base.to_owned()
            } else {
                format!("{base} {i}")
            }
        })
        .find(free)
        .unwrap_or_else(|| base.to_owned())
}

#[tauri::command]
pub fn add_calendar(app: State<'_, App>, url: String, area: String) -> AppResult<()> {
    let settings = app.settings()?;
    let area = settings.known_area(&area);
    let label = settings
        .areas
        .iter()
        .find(|a| a.id == area)
        .map_or_else(|| area.clone(), |a| a.name.clone());
    let feed = Feed {
        name: free_name(&settings.calendar.feeds, &label),
        url: https(&url)?,
        area,
        ignore_titles: Vec::new(),
    };
    edit::upsert_feed(&app.config, &feed)?;
    app.db.with(|c| status::sync_soon(c, calendar::SOURCE))
}

#[tauri::command]
pub fn remove_calendar(app: State<'_, App>, name: String) -> AppResult<()> {
    edit::remove_feed(&app.config, &name)?;
    app.db.with(|c| status::sync_soon(c, calendar::SOURCE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_and_names() {
        assert_eq!(https(" webcal://x/a.ics ").unwrap(), "https://x/a.ics");
        assert!(https("ftp://x").is_err());
        let taken = [Feed {
            name: "Work".to_owned(),
            url: String::new(),
            area: "work".to_owned(),
            ignore_titles: Vec::new(),
        }];
        assert_eq!(free_name(&taken, "Work"), "Work 2");
        assert_eq!(free_name(&taken, "Life"), "Life");
    }
}
