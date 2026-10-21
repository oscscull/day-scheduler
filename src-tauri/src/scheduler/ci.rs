//! CI results go to whoever is on the work, so no agent ever polls CI. A Claude session that
//! claimed the item is resumed with the result. A failure nobody is on is handed to a new Claude
//! session, which is claimed so later results reach it too.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::claude::{self, Ask, Session};
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::connectors::github::Fired;
use crate::domain::item::WorkItem;
use crate::domain::run::{Outcome, Run};
use crate::domain::schedule::EventKind;
use crate::error::AppResult;
use crate::store::{items, runs, watch};

/// One Claude run Bridge starts about an item.
struct Job {
    what: &'static str,
    item: WorkItem,
    prompt: String,
    session: String,
    resume: bool,
}

/// Reacts to a CI event. `covered` is true when a schedule already handles it.
pub fn react(
    handle: &AppHandle,
    settings: &Settings,
    event: &Fired,
    covered: bool,
) -> AppResult<()> {
    let app = handle.state::<App>();
    let item = app.db.with(|c| items::get(c, &event.item_id))?;
    let claimed = app.db.with(watch::all)?.remove(&item.id);
    if let Some(session) = claimed.and_then(|w| w.session) {
        let result = if event.kind == EventKind::CiPassed {
            "passed"
        } else {
            "failed"
        };
        let prompt = prompt_for(&app, Prompt::CiDone, &item, result)?;
        return run(
            handle,
            settings,
            Job {
                what: "CI result",
                item,
                prompt,
                session,
                resume: true,
            },
        );
    }
    if event.kind == EventKind::CiFailed && settings.github.fix_ci && !covered {
        let session = uuid::Uuid::new_v4().to_string();
        app.db.with(|c| watch::claim(c, &item.id, Some(&session)))?;
        let prompt = prompt_for(&app, Prompt::FixCi, &item, "failed")?;
        return run(
            handle,
            settings,
            Job {
                what: "Fix CI",
                item,
                prompt,
                session,
                resume: false,
            },
        );
    }
    Ok(())
}

fn prompt_for(app: &App, prompt: Prompt, item: &WorkItem, result: &str) -> AppResult<String> {
    let link = item.link.clone().unwrap_or_default();
    let vars = [
        ("item", item.id.as_str()),
        ("title", item.title.as_str()),
        ("link", link.as_str()),
        ("result", result),
    ];
    render(&app.config, prompt, &vars)
}

fn start_record(handle: &AppHandle, settings: &Settings, job: &Job) -> AppResult<i64> {
    let record = Run {
        id: 0,
        schedule_id: None,
        what: job.what.to_owned(),
        target: job.item.id.clone(),
        node: settings.node.clone(),
        started_at: Utc::now(),
        finished_at: None,
        outcome: None,
        summary: None,
    };
    handle.state::<App>().db.with(|c| runs::start(c, &record))
}

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect()
}

fn run(handle: &AppHandle, settings: &Settings, job: Job) -> AppResult<()> {
    let id = start_record(handle, settings, &job)?;
    let command = settings.claude_command.clone();
    let tools = settings.github.fix_ci_tools.clone();
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let session = if job.resume {
            Session::Resume(&job.session)
        } else {
            Session::Start(&job.session)
        };
        let reply = claude::ask_with(
            &command,
            &job.prompt,
            &Ask {
                tools: &tools,
                session,
            },
        )
        .await;
        let (outcome, summary) = match reply {
            Ok(text) => (Outcome::Ok, first_line(&text)),
            Err(e) => (Outcome::Failed, e.to_string()),
        };
        // The run record is for display; losing it changes nothing about the item.
        drop(
            handle
                .state::<App>()
                .db
                .with(|c| runs::finish(c, id, outcome, &summary)),
        );
    });
    Ok(())
}
