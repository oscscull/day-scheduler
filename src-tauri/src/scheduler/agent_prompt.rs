//! Prompting the agent on its turn when nothing is running: the session holding the item carries
//! on, or a new one starts in the item's worktree (or Quarterdeck's folder, with no project). A
//! retry asked for from an alert sends the failed run's prompt instead.

use tauri::{AppHandle, Manager as _};

use super::dispatch::{self, Job};
use super::thread::{self, Thread};
use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::item::WorkItem;
use crate::error::AppResult;
use crate::retry;
use crate::store::watch;
use crate::turns;
use crate::workspace::{self, Workspace};

/// The session to carry on in, and the worktree it runs in.
async fn thread(
    app: &App,
    settings: &Settings,
    item: &WorkItem,
) -> AppResult<(Thread, Option<Workspace>)> {
    let ws = workspace::for_item(settings, item)?;
    let known = watch::session_for(&app.db, &item.id).await?;
    let thread = thread::thread_for(known, ws.as_ref())
        .unwrap_or_else(|| Thread::Start(thread::fresh_session()));
    Ok((thread, ws))
}

async fn prompt_for(app: &App, item: &WorkItem, ws: Option<&Workspace>) -> AppResult<String> {
    let why = turns::history::last_why(&app.db, &item.id)
        .await?
        .unwrap_or_default();
    let link = item.link.clone().unwrap_or_default();
    let place = workspace::describe(&app.config, ws)?;
    let vars = [
        ("item", item.id.as_str()),
        ("title", item.title.as_str()),
        ("link", link.as_str()),
        ("label", item.state_label.as_str()),
        ("why", why.as_str()),
        ("workspace", place.as_str()),
    ];
    render(&app.config, Prompt::Continue, &vars)
}

/// The failed run's prompt when a retry was asked for, otherwise "carry on".
async fn job_for(
    app: &App,
    item: &WorkItem,
    thread: Thread,
    ws: Option<&Workspace>,
) -> AppResult<Job> {
    if let Some(asked) = retry::take(&app.db, &item.id).await? {
        return Ok(Job::new(&asked.what, item.id.clone(), asked.prompt, thread));
    }
    let prompt = prompt_for(app, item, ws).await?;
    Ok(Job::new("Carry on", item.id.clone(), prompt, thread))
}

pub async fn carry_on(handle: &AppHandle, settings: &Settings, item: &WorkItem) -> AppResult<()> {
    // A build that may not run the agent would only fail the same run every tick.
    if settings.claude().is_err() {
        return Ok(());
    }
    let app = handle.state::<App>();
    let (thread, ws) = thread(&app, settings, item).await?;
    let job = job_for(&app, item, thread, ws.as_ref())
        .await?
        .in_workspace(ws);
    dispatch::run(handle, settings, job).await
}
