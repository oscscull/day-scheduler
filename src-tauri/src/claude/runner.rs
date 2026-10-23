//! Runs one schedule through headless Claude and records the outcome.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::domain::activity::Activity;
use crate::domain::fired::Fired;
use crate::domain::run::{Outcome, Run};
use crate::domain::schedule::Schedule;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{activity, fired, runs, urgent};

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

fn started(schedule: &Schedule, node: &str, cause: Option<&Fired>) -> Run {
    Run {
        id: 0,
        schedule_id: Some(schedule.id.clone()),
        what: schedule.name.clone(),
        target: cause.map(|c| c.subject.clone()).unwrap_or_default(),
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
        run_id: None,
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

/// Where `ask` runs: on `node`, logging its steps under run `id`, `depth` events down a chain.
struct Place<'a> {
    node: &'a str,
    id: i64,
    depth: u32,
}

/// Why it's running, as the prompt says it: the event's line and its fields.
fn why(cause: Option<&Fired>) -> AppResult<String> {
    let Some(event) = cause else {
        return Ok("its schedule came due, or someone asked to run it now".to_owned());
    };
    let fields = serde_json::to_string(&event.data)?;
    Ok(format!(
        "{} (event {}, fields {fields})",
        event.about, event.kind
    ))
}

/// What other schedules can react to.
fn finished(schedule: &Schedule, outcome: Outcome, summary: &str, depth: u32) -> Fired {
    let word = match outcome {
        Outcome::Ok => "ok",
        Outcome::Failed | Outcome::Missed => "failed",
    };
    let about = format!("{} finished ({word}): {summary}", schedule.name);
    Fired::new("run.finished", &schedule.area, &schedule.id, about)
        .with("schedule", &schedule.id)
        .with("outcome", word)
        .at_depth(depth)
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
        depth: place.depth,
    };
    super::ask_with(settings.claude()?, &prompt, &opts).await
}

/// Runs `schedule` now. Success lands in "From Claude"; failure is raised as urgent.
/// Both send a quiet notification.
/// `cause` is the event that started it, if one did, e.g. new comments on a PR. What it does
/// counts as one event further down that event's chain.
pub async fn run(handle: &AppHandle, schedule: &Schedule, cause: Option<&Fired>) -> AppResult<()> {
    let app = handle.state::<App>();
    let node = app.settings()?.node_for(&schedule.area).to_owned();
    let id = runs::start(&app.db, &started(schedule, &node, cause)).await?;
    let depth = cause.map_or(0, |c| c.depth) + 1;
    let place = Place {
        node: &node,
        id,
        depth,
    };
    let (outcome, summary) = match ask(&app, schedule, &place, &why(cause)?).await {
        Ok(text) => (Outcome::Ok, summary_of(&text)),
        Err(e) => (Outcome::Failed, summary_of(&e.to_string())),
    };
    runs::finish(&app.db, id, outcome, &summary).await?;
    fired::emit(&app.db, &finished(schedule, outcome, &summary, depth)).await?;
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
