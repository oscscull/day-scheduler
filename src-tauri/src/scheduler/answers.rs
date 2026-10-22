//! Takes the user's answers back to Claude. Work on a PR in a known project happens in that
//! project's worktree: the asking session is resumed if it ran there, otherwise a new session
//! starts there with the question and answer. Elsewhere the session that asked, or the one holding
//! the item, is resumed. With nobody to tell, the answer just stays on the item's history.

use std::collections::HashSet;

use tauri::{AppHandle, Manager as _};

use super::dispatch::{self, Job, Thread};
use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::ask::Answered;
use crate::domain::item::WorkItem;
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::store::{asks, items, runs, watch};
use crate::workspace::{self, Workspace};

pub async fn deliver(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let waiting = asks::answered(&app.db).await?;
    // One run per item at a time: a second answer waits for the first run, then resumes its session.
    let mut busy: HashSet<String> = runs::live(&app.db)
        .await?
        .into_iter()
        .map(|r| r.target)
        .collect();
    for answered in waiting {
        if busy.contains(&answered.ask.item_id) {
            continue;
        }
        let item = items::get(&app.db, &answered.ask.item_id).await?;
        // Another machine runs this area's agents and delivers there.
        if settings.node_for(&item.area) != settings.node {
            continue;
        }
        let known = known_session(&app, &answered, &item).await?;
        let ws = workspace::for_item(settings, &item)?;
        if let Some(thread) = thread_for(known, ws.as_ref()) {
            if let Thread::Start(session) = &thread {
                watch::claim(&app.db, &item.id, Some(session)).await?;
            }
            let prompt = prompt_for(&app, &answered, &item, ws.as_ref())?;
            let job = Job::new("Answer", item.id.clone(), prompt, thread).in_workspace(ws);
            dispatch::run(handle, settings, job).await?;
            busy.insert(item.id.clone());
        }
        asks::remove(&app.db, answered.ask.id).await?;
    }
    Ok(())
}

/// The session that asked, else the one holding the item.
async fn known_session(
    app: &App,
    answered: &Answered,
    item: &WorkItem,
) -> AppResult<Option<SessionRef>> {
    if answered.ask.asker.is_some() {
        return Ok(answered.ask.asker.clone());
    }
    Ok(watch::all(&app.db)
        .await?
        .remove(&item.id)
        .and_then(|w| w.session))
}

fn thread_for(known: Option<SessionRef>, ws: Option<&Workspace>) -> Option<Thread> {
    match (known, ws) {
        (Some(s), Some(ws)) if ws.holds(&s) => Some(Thread::Resume(s)),
        (_, Some(ws)) => Some(Thread::Start(ws.new_session())),
        (Some(s), None) => Some(Thread::Resume(s)),
        (None, None) => None,
    }
}

fn prompt_for(
    app: &App,
    answered: &Answered,
    item: &WorkItem,
    ws: Option<&Workspace>,
) -> AppResult<String> {
    let link = item.link.clone().unwrap_or_default();
    let place = workspace::describe(&app.config, ws)?;
    let vars = [
        ("item", item.id.as_str()),
        ("title", item.title.as_str()),
        ("link", link.as_str()),
        ("question", answered.ask.question.as_str()),
        ("context", answered.ask.context.as_str()),
        ("answer", answered.answer.as_str()),
        ("workspace", place.as_str()),
    ];
    let prompt = if answered.ask.question.is_empty() {
        Prompt::Message
    } else {
        Prompt::Answer
    };
    render(&app.config, prompt, &vars)
}
