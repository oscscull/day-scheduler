//! An agent run about an item that failed, and how the failure is handed back to the user.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::turns::TurnSettings;
use crate::domain::item::WorkItem;
use crate::domain::turn::Turn;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{Db, items, urgent, watch};
use crate::turns::{self, Clock, Pass};

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

async fn hand_back(db: &Db, f: &Failure<'_>, limits: &TurnSettings) -> AppResult<String> {
    watch::release(db, f.item_id).await?;
    let item = items::get(db, f.item_id).await?;
    let title = raise_urgent(db, &item, f).await?;
    let why = format!("your agent couldn't run ({}): {}", f.what, f.detail());
    turns::pass(db, &item.id, Pass::to(Turn::You, why), Clock::now(limits)).await?;
    Ok(title)
}

pub async fn report(
    handle: &AppHandle,
    failure: &Failure<'_>,
    limits: &TurnSettings,
) -> AppResult<()> {
    let title = hand_back(&handle.state::<App>().db, failure, limits).await?;
    notify::send(handle, &title, &failure.brief())
}
