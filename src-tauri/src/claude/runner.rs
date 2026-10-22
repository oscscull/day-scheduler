//! Runs one schedule through headless Claude and records the outcome.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::domain::activity::Activity;
use crate::domain::run::{Outcome, Run};
use crate::domain::schedule::Schedule;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{activity, runs, urgent};

const SUMMARY_CHARS: usize = 200;

/// Claude's first line, kept short: run history is for glancing, not reading.
fn summary_of(reply: &str) -> String {
    reply
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .chars()
        .take(SUMMARY_CHARS)
        .collect()
}

fn started(schedule: &Schedule, node: &str) -> Run {
    Run {
        id: 0,
        schedule_id: Some(schedule.id.clone()),
        what: schedule.name.clone(),
        target: String::new(),
        node: node.to_owned(),
        started_at: Utc::now(),
        finished_at: None,
        outcome: None,
        summary: None,
    }
}

fn failure(schedule: &Schedule, detail: String) -> Urgent {
    Urgent {
        id: 0,
        title: format!("{} failed", schedule.name),
        detail,
        area: schedule.area.clone(),
        raised_at: Utc::now(),
        schedule_id: Some(schedule.id.clone()),
    }
}

fn note(schedule: &Schedule, summary: &str) -> Activity {
    Activity {
        id: 0,
        text: format!("{}: {summary}", schedule.name),
        area: schedule.area.clone(),
        at: Utc::now(),
        unread: true,
    }
}

/// Where `ask` runs: on `node`, logging its steps under run `id`.
struct Place<'a> {
    node: &'a str,
    id: i64,
}

async fn ask(
    app: &App,
    schedule: &Schedule,
    place: &Place<'_>,
    trigger: &str,
) -> AppResult<String> {
    let settings = app.settings()?;
    let vars = [
        ("node", place.node),
        ("name", schedule.name.as_str()),
        ("instructions", schedule.prompt.as_str()),
        ("trigger", trigger),
    ];
    let prompt = render(&app.config, Prompt::RunSchedule, &vars)?;
    let opts = super::Ask {
        tools: &schedule.allowed_tools,
        session: super::Session::Fresh,
        cwd: None,
        permission_mode: None,
        run: Some(place.id),
    };
    super::ask_with(&settings.claude_command, &prompt, &opts).await
}

/// Runs `schedule` now. Success lands in "From Claude"; failure is raised as urgent.
/// Both send a quiet notification.
/// `trigger` says why it ran, e.g. which PR got new comments.
pub async fn run(handle: &AppHandle, schedule: &Schedule, trigger: Option<&str>) -> AppResult<()> {
    let app = handle.state::<App>();
    let node = app.settings()?.node_for(&schedule.area).to_owned();
    let id = runs::start(&app.db, &started(schedule, &node)).await?;
    let why = trigger.unwrap_or("its schedule came due, or someone asked to run it now");
    let (outcome, summary) = match ask(&app, schedule, &Place { node: &node, id }, why).await {
        Ok(text) => (Outcome::Ok, summary_of(&text)),
        Err(e) => (Outcome::Failed, summary_of(&e.to_string())),
    };
    runs::finish(&app.db, id, outcome, &summary).await?;
    match outcome {
        Outcome::Ok => activity::insert(&app.db, &note(schedule, &summary)).await?,
        Outcome::Failed | Outcome::Missed => {
            urgent::raise(&app.db, &failure(schedule, summary.clone())).await?;
        }
    }
    let title = match outcome {
        Outcome::Ok => schedule.name.clone(),
        Outcome::Failed | Outcome::Missed => format!("{} failed", schedule.name),
    };
    // Notifications are a courtesy; the outcome is already recorded.
    drop(notify::send(handle, &title, &summary));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_the_first_real_line() {
        assert_eq!(
            summary_of("\n  \nDid the thing.\nMore detail."),
            "Did the thing."
        );
        assert_eq!(summary_of(&"x".repeat(500)).len(), SUMMARY_CHARS);
    }
}
