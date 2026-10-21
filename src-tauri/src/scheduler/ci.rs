//! CI results go to whoever is on the work, so no agent ever polls CI. A Claude session that
//! claimed the item is resumed with the result. A failure nobody is on is handed to a new Claude
//! session, which is claimed so later results reach it too.

use tauri::{AppHandle, Manager as _};

use super::dispatch::{self, Job, Thread};
use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::connectors::github::Fired;
use crate::domain::item::WorkItem;
use crate::domain::schedule::EventKind;
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::store::{items, watch};

/// Reacts to a CI event. `covered` is true when a schedule already handles it.
pub fn react(
    handle: &AppHandle,
    settings: &Settings,
    event: &Fired,
    covered: bool,
) -> AppResult<()> {
    let app = handle.state::<App>();
    let item = app.db.with(|c| items::get(c, &event.item_id))?;
    let tools = settings.github.fix_ci_tools.clone();
    let claimed = app.db.with(watch::all)?.remove(&item.id);
    if let Some(session) = claimed.and_then(|w| w.session) {
        let result = if event.kind == EventKind::CiPassed {
            "passed"
        } else {
            "failed"
        };
        let prompt = prompt_for(&app, Prompt::CiDone, &item, result)?;
        let job = Job {
            what: "CI result",
            item_id: item.id,
            prompt,
            thread: Thread::Resume(session),
            tools,
        };
        return dispatch::run(handle, settings, job);
    }
    if event.kind == EventKind::CiFailed && settings.github.fix_ci && !covered {
        let session = SessionRef {
            id: uuid::Uuid::new_v4().to_string(),
            cwd: None,
        };
        app.db.with(|c| watch::claim(c, &item.id, Some(&session)))?;
        let prompt = prompt_for(&app, Prompt::FixCi, &item, "failed")?;
        let job = Job {
            what: "Fix CI",
            item_id: item.id,
            prompt,
            thread: Thread::Start(session),
            tools,
        };
        return dispatch::run(handle, settings, job);
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
