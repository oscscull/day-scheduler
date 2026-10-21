//! Takes the user's answers back to the session that asked, or failing that the session working on
//! the item. Answers are already on the item's history, so with nobody to tell they just clear.

use tauri::{AppHandle, Manager as _};

use super::dispatch::{self, Job, Thread};
use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::ask::Answered;
use crate::error::AppResult;
use crate::store::{asks, items, watch};

pub fn deliver(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    let app = handle.state::<App>();
    let waiting = app.db.with(asks::answered)?;
    if waiting.is_empty() {
        return Ok(());
    }
    let watches = app.db.with(watch::all)?;
    for answered in waiting {
        let item = app.db.with(|c| items::get(c, &answered.ask.item_id))?;
        // Another machine runs this area's agents and delivers there.
        if settings.node_for(&item.area) != settings.node {
            continue;
        }
        let working = watches.get(&item.id).and_then(|w| w.session.clone());
        if let Some(session) = answered.ask.asker.clone().or(working) {
            let job = Job {
                what: "Answer",
                item_id: item.id.clone(),
                prompt: prompt_for(&app, &answered)?,
                thread: Thread::Resume(session),
                tools: Vec::new(),
            };
            dispatch::run(handle, settings, job)?;
        }
        app.db.with(|c| asks::remove(c, answered.ask.id))?;
    }
    Ok(())
}

fn prompt_for(app: &App, answered: &Answered) -> AppResult<String> {
    let vars = [
        ("item", answered.ask.item_id.as_str()),
        ("question", answered.ask.question.as_str()),
        ("answer", answered.answer.as_str()),
    ];
    render(&app.config, Prompt::Answer, &vars)
}
