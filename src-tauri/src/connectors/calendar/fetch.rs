//! Gets a feed's events, from its private link or the macOS Calendar app.

use chrono::{DateTime, Utc};
use tokio::process::Command;

use super::macos;
use super::parse::{self, Occurrence};
use crate::cli;
use crate::config::calendar::{Feed, Origin};
use crate::error::{AppError, AppResult};

/// The feed's occurrences between `from` and `to`.
pub async fn occurrences(
    feed: &Feed,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> AppResult<Vec<Occurrence>> {
    match &feed.origin {
        Origin::Link { url } => parse::occurrences(&ics(url).await?, from, to),
        Origin::Mac { mac_calendar } => {
            let name = mac_calendar.clone();
            tauri::async_runtime::spawn_blocking(move || macos::occurrences(&name, from, to))
                .await?
        }
    }
}

/// Downloads a feed with the system's curl. `webcal://` links are fetched over https.
async fn ics(url: &str) -> AppResult<String> {
    let url = url
        .strip_prefix("webcal://")
        .map_or_else(|| url.to_owned(), |rest| format!("https://{rest}"));
    let out = Command::new(cli::resolve("curl"))
        .args(["-fsSL", "--max-time", "30", &url])
        .output()
        .await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(AppError::Connector(format!(
            "couldn't fetch a calendar: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )))
    }
}
