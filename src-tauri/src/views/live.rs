//! The runs going right now, each with the item it's about and what it did last.

use serde::Serialize;

use crate::domain::run::Run;
use crate::domain::step::Step;
use crate::error::AppResult;
use crate::runlog;
use crate::store::{Db, items, runs};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveRun {
    #[serde(flatten)]
    pub run: Run,
    /// The title of the item it works on, if it works on one.
    pub title: Option<String>,
    /// Its latest step, once it has taken one.
    pub now: Option<Step>,
}

async fn title_of(db: &Db, target: &str) -> AppResult<Option<String>> {
    if target.is_empty() {
        return Ok(None);
    }
    Ok(items::find(db, target).await?.map(|i| i.title))
}

pub async fn build(db: &Db) -> AppResult<Vec<LiveRun>> {
    // Without the folder there are no steps to show; the runs still are.
    let dir = runlog::dir().ok();
    let mut live = Vec::new();
    for run in runs::live(db).await? {
        let title = title_of(db, &run.target).await?;
        let now = dir.as_deref().and_then(|d| runlog::latest(d, run.id));
        live.push(LiveRun { run, title, now });
    }
    Ok(live)
}
