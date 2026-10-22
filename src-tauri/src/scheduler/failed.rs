//! An agent run about an item that failed. Nobody is working on the item, so it goes back to the
//! user loudly: urgent, a notification, a line in its history, and their turn again.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Status};
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{Db, events, items, urgent, watch};
use crate::work;

pub struct Failure<'a> {
    pub item_id: &'a str,
    /// What the run was for, e.g. "Answer".
    pub what: &'a str,
    pub detail: &'a str,
    /// The run was to start a session, which now doesn't exist and mustn't be resumed.
    pub started_session: bool,
}

async fn hand_back(db: &Db, f: &Failure<'_>) -> AppResult<String> {
    if f.started_session {
        watch::release(db, f.item_id).await?;
    }
    let item = items::get(db, f.item_id).await?;
    let now = Utc::now();
    let text = format!("Your agent couldn't run ({}): {}", f.what, f.detail);
    events::record(db, &item.id, now, &Change::Note { text }).await?;
    let title = format!("Your agent couldn't run on {}", item.id);
    let raised = Urgent {
        id: 0,
        title: title.clone(),
        detail: f.detail.to_owned(),
        area: item.area.clone(),
        raised_at: now,
        schedule_id: None,
    };
    urgent::raise(db, &raised).await?;
    let input = ItemInput {
        id: item.id,
        title: item.title,
        state_label: "agent couldn't run".to_owned(),
        stage: item.stage,
        status: Some(Status::Decision),
        holder: "you".to_owned(),
        area: item.area,
        link: item.link,
    };
    work::save(db, input, now).await?;
    Ok(title)
}

pub async fn report(handle: &AppHandle, failure: &Failure<'_>) -> AppResult<()> {
    let title = hand_back(&handle.state::<App>().db, failure).await?;
    notify::send(handle, &title, failure.detail)
}
