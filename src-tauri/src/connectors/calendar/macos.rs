//! Calendars in the macOS Calendar app, read through the `quarterdeck-calendars` helper that ships
//! beside the app's binary (`helpers/calendars.swift`). The Calendar app handles each account's
//! sign-in and expands repeating events, so this works where an account offers no iCal link.

use std::path::PathBuf;
use std::process::Command;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use super::links::first_call_link;
use super::parse::Occurrence;
use crate::error::{AppError, AppResult};

const HELPER: &str = "quarterdeck-calendars";

/// A calendar the Calendar app shows, and the account it belongs to.
#[derive(Debug, Serialize, Deserialize)]
pub struct MacCalendar {
    pub title: String,
    pub account: String,
}

#[derive(Debug, Deserialize)]
struct Event {
    id: String,
    title: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    all_day: bool,
    location: Option<String>,
    notes: Option<String>,
    url: Option<String>,
}

fn helper() -> AppResult<PathBuf> {
    std::env::current_exe()?
        .parent()
        .map(|dir| dir.join(HELPER))
        .filter(|path| path.exists())
        .ok_or_else(|| AppError::Connector("the Calendar app can only be read on a Mac".to_owned()))
}

/// Runs the helper. The first run may wait while macOS asks the user for calendar access.
fn run(args: &[&str]) -> AppResult<Vec<u8>> {
    let out = Command::new(helper()?).args(args).output()?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(AppError::Connector(
            String::from_utf8_lossy(&out.stderr).trim().to_owned(),
        ))
    }
}

pub fn calendars() -> AppResult<Vec<MacCalendar>> {
    Ok(serde_json::from_slice(&run(&["list"])?)?)
}

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// The occurrences of the calendar called `name` between `from` and `to`.
pub fn occurrences(
    name: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> AppResult<Vec<Occurrence>> {
    let json = run(&["events", name, &rfc3339(from), &rfc3339(to)])?;
    let events: Vec<Event> = serde_json::from_slice(&json)?;
    Ok(events.into_iter().map(occurrence).collect())
}

fn occurrence(e: Event) -> Occurrence {
    let join_url = [&e.url, &e.location, &e.notes]
        .into_iter()
        .flatten()
        .find_map(|text| first_call_link(text));
    Occurrence {
        id: e.id,
        title: e.title,
        start: e.start,
        end: e.end,
        location: e.location.filter(|l| !l.is_empty()),
        join_url,
        all_day: e.all_day,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_helper_events_with_their_call_links() {
        let json = r#"[{"id":"abc@2026-10-22T01:00:00Z","title":"Planning","start":"2026-10-22T01:00:00Z",
            "end":"2026-10-22T01:30:00Z","all_day":false,"location":"",
            "notes":"Join with Google Meet: https://meet.google.com/abc-defg-hij\nMore phone numbers"}]"#;
        let events: Vec<Event> = serde_json::from_str(json).unwrap();
        let o = occurrence(events.into_iter().next().unwrap());
        assert_eq!(o.end - o.start, chrono::Duration::minutes(30));
        assert_eq!(
            o.join_url.as_deref(),
            Some("https://meet.google.com/abc-defg-hij")
        );
        assert_eq!(o.location, None);
    }
}
