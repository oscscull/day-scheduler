//! An agent run about an item that failed, and how the failure is handed back to the user.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Status, WorkItem};
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
}

/// What to show when a run died without saying why.
const NO_DETAIL: &str = "it stopped without saying why";
const NEXT_STEP: &str = "Open its run on the Agent tab, or tell Claude to look into it.";
/// The longest reason an alert shows; the item's history keeps all of it.
const BRIEF_CHARS: usize = 200;

impl Failure<'_> {
    fn detail(&self) -> &str {
        if self.detail.trim().is_empty() {
            NO_DETAIL
        } else {
            self.detail
        }
    }

    /// The first line of the reason and what to do, for an alert.
    fn brief(&self) -> String {
        let first: String = self
            .detail()
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(BRIEF_CHARS)
            .collect();
        format!("{first}. {NEXT_STEP}")
    }
}

async fn raise_urgent(db: &Db, item: &WorkItem, f: &Failure<'_>) -> AppResult<String> {
    let title = format!("Your agent couldn't run on {}", item.id);
    let raised = Urgent {
        id: 0,
        title: title.clone(),
        detail: f.brief(),
        area: item.area.clone(),
        raised_at: Utc::now(),
        schedule_id: None,
    };
    urgent::raise(db, &raised).await?;
    Ok(title)
}

async fn return_to_user(db: &Db, item: WorkItem) -> AppResult<()> {
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
    work::save(db, input, Utc::now()).await.map(drop)
}

async fn hand_back(db: &Db, f: &Failure<'_>) -> AppResult<String> {
    watch::release(db, f.item_id).await?;
    let item = items::get(db, f.item_id).await?;
    let text = format!("Your agent couldn't run ({}): {}", f.what, f.detail());
    events::record(db, &item.id, Utc::now(), &Change::Note { text }).await?;
    let title = raise_urgent(db, &item, f).await?;
    return_to_user(db, item).await?;
    Ok(title)
}

pub async fn report(handle: &AppHandle, failure: &Failure<'_>) -> AppResult<()> {
    let title = hand_back(&handle.state::<App>().db, failure).await?;
    notify::send(handle, &title, &failure.brief())
}
