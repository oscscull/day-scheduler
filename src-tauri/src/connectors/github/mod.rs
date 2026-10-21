//! GitHub: the user's open PRs become items, and changes on them fire event schedules.

mod apply;
mod fetch;
mod snapshot;

use std::collections::HashSet;

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::domain::schedule::EventKind;
use crate::error::AppResult;

pub use apply::SOURCE;

/// One event on one PR, for starting event schedules.
#[derive(Debug, Clone)]
pub struct Fired {
    pub kind: EventKind,
    pub item_area: String,
    /// Told to the run, e.g. "new review comments on me/app#7 (https://…)".
    pub trigger: String,
}

fn describe(kind: EventKind, ext: &str, url: &str) -> String {
    let what = match kind {
        EventKind::PrNewComments => "new review comments",
        EventKind::CiFailed => "CI failing",
    };
    format!("{what} on {ext} ({url})")
}

/// Pulls open PRs into items and returns the events they fired.
pub async fn sync(handle: &AppHandle) -> AppResult<Vec<Fired>> {
    let app = handle.state::<App>();
    let settings = app.settings()?.github;
    let prs = fetch::open_prs().await?;
    let now = Utc::now();
    app.db.with(|conn| {
        let mut fired = Vec::new();
        let mut open = HashSet::new();
        for pr in prs.iter().filter(|p| !settings.ignores(&p.repo)) {
            let area = settings.area_for(&pr.repo);
            let ext = apply::external_id(pr);
            for kind in apply::apply(conn, pr, area, now)? {
                fired.push(Fired {
                    kind,
                    item_area: area.to_owned(),
                    trigger: describe(kind, &ext, &pr.url),
                });
            }
            open.insert(ext);
        }
        apply::close_missing(conn, &open, now)?;
        Ok(fired)
    })
}
