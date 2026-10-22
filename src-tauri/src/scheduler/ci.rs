//! CI results go to whoever is on the work, so no agent ever polls CI. A Claude session that
//! claimed the item is resumed with the result. A failure nobody is on is handed to a new Claude
//! session, in the repo's project when there is one, and claimed so later results reach it too.

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
use crate::workspace::{self, Workspace};

/// Reacts to a CI event. `covered` is true when a schedule already handles it.
pub async fn react(
    handle: &AppHandle,
    settings: &Settings,
    event: &Fired,
    covered: bool,
) -> AppResult<()> {
    let app = handle.state::<App>();
    let item = items::get(&app.db, &event.item_id).await?;
    let ws = workspace::for_item(settings, &item)?;
    let claimed = watch::all(&app.db).await?.remove(&item.id);
    let job = match claimed.and_then(|w| w.session) {
        Some(session) => {
            let passed = event.kind == EventKind::CiPassed;
            let result = if passed { "passed" } else { "failed" };
            let prompt = prompt_for(&app, Prompt::CiDone, &item, &[("result", result)])?;
            let ws = ws.filter(|w| w.holds(&session));
            Job::new("CI result", item.id, prompt, Thread::Resume(session)).in_workspace(ws)
        }
        None if event.kind == EventKind::CiFailed && settings.github.fix_ci && !covered => {
            let session = ws.as_ref().map_or_else(fresh, Workspace::new_session);
            watch::claim(&app.db, &item.id, Some(&session)).await?;
            let place = workspace::describe(&app.config, ws.as_ref())?;
            let prompt = prompt_for(&app, Prompt::FixCi, &item, &[("workspace", place.as_str())])?;
            Job::new("Fix CI", item.id, prompt, Thread::Start(session)).in_workspace(ws)
        }
        None => return Ok(()),
    };
    dispatch::run(
        handle,
        settings,
        job.with_tools(&settings.github.fix_ci_tools),
    )
    .await
}

/// A new session in Quarterdeck's own folder, for repos with no project here.
fn fresh() -> SessionRef {
    SessionRef {
        id: uuid::Uuid::new_v4().to_string(),
        cwd: None,
    }
}

/// `extra` adds `result` and `workspace` to the item's own fields.
fn prompt_for(
    app: &App,
    prompt: Prompt,
    item: &WorkItem,
    extra: &[(&str, &str)],
) -> AppResult<String> {
    let link = item.link.clone().unwrap_or_default();
    let mut vars = vec![
        ("item", item.id.as_str()),
        ("title", item.title.as_str()),
        ("link", link.as_str()),
    ];
    vars.extend_from_slice(extra);
    render(&app.config, prompt, &vars)
}
