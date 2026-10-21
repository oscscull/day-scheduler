//! Takes the user's answers back to Claude. Work on a PR or issue in a known project happens in that
//! project's worktree: the asking session is resumed if it ran there, otherwise a new session
//! starts there with the question and answer. Elsewhere the session that asked, or the one holding
//! the item, is resumed. With nobody to tell, the user is told instead (see `undelivered`).

use std::collections::HashSet;

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use super::dispatch::{self, Job, Thread};
use super::undelivered;
use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::ask::Answered;
use crate::domain::item::WorkItem;
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::store::{asks, items, runs, watch};
use crate::workspace::{self, Workspace};

pub fn deliver(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let waiting = app.db.with(asks::answered)?;
    // One run per item at a time: a second answer waits for the first run, then resumes its session.
    let mut busy: HashSet<String> = app
        .db
        .with(runs::live)?
        .into_iter()
        .map(|r| r.target)
        .collect();
    for answered in waiting {
        if busy.contains(&answered.ask.item_id) {
            continue;
        }
        let item = app.db.with(|c| items::get(c, &answered.ask.item_id))?;
        // Another machine runs this area's agents and delivers there.
        if settings.node_for(&item.area) != settings.node {
            continue;
        }
        let known = known_session(&app, &answered, &item)?;
        let ws = workspace::for_item(settings, &item)?;
        match thread_for(known, ws.as_ref()) {
            Some(thread) => {
                start(handle, settings, (&answered, &item), (thread, ws))?;
                busy.insert(item.id.clone());
            }
            None => app
                .db
                .with(|c| undelivered::report(c, &answered, Utc::now()))?,
        }
        app.db.with(|c| asks::remove(c, answered.ask.id))?;
    }
    Ok(())
}

/// Hands the answer to Claude, claiming the item for a new session.
fn start(
    handle: &AppHandle,
    settings: &Settings,
    (answered, item): (&Answered, &WorkItem),
    (thread, ws): (Thread, Option<Workspace>),
) -> AppResult<()> {
    let app = handle.state::<App>();
    if let Thread::Start(session) = &thread {
        app.db.with(|c| watch::claim(c, &item.id, Some(session)))?;
    }
    let prompt = prompt_for(&app, answered, item, ws.as_ref())?;
    let job = Job::new("Answer", item.id.clone(), prompt, thread).in_workspace(ws);
    dispatch::run(handle, settings, job)
}

/// The session that asked, else the one holding the item.
fn known_session(app: &App, answered: &Answered, item: &WorkItem) -> AppResult<Option<SessionRef>> {
    if answered.ask.asker.is_some() {
        return Ok(answered.ask.asker.clone());
    }
    Ok(app
        .db
        .with(watch::all)?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::Kind;

    fn session(cwd: Option<&str>) -> SessionRef {
        SessionRef {
            id: "s".to_owned(),
            cwd: cwd.map(str::to_owned),
        }
    }

    #[test]
    fn only_items_with_no_session_and_no_workspace_have_nobody_to_tell() {
        let ws = Workspace {
            project: "/code/app".into(),
            repo: "me/app".to_owned(),
            number: 3,
            kind: Kind::Issue,
            dir: "/wt/app-3".into(),
        };
        assert!(thread_for(None, None).is_none());
        assert!(matches!(
            thread_for(Some(session(None)), None),
            Some(Thread::Resume(_))
        ));
        assert!(matches!(
            thread_for(None, Some(&ws)),
            Some(Thread::Start(_))
        ));
        assert!(matches!(
            thread_for(Some(session(Some("/wt/app-3"))), Some(&ws)),
            Some(Thread::Resume(_))
        ));
    }
}
