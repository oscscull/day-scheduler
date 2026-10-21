//! GitHub: the user's open PRs become items, and changes on them fire event schedules.

mod apply;
mod checks;
mod fetch;
mod snapshot;

use std::collections::HashSet;

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::domain::schedule::EventKind;
use crate::error::AppResult;
use crate::store::items;

pub use apply::SOURCE;

/// One event on one PR, for starting event schedules.
#[derive(Debug, Clone)]
pub struct Fired {
    pub kind: EventKind,
    pub item_id: String,
    pub item_area: String,
    /// Told to the run, e.g. "new review comments on me/app#7 (https://…)".
    pub trigger: String,
}

fn describe(kind: EventKind, ext: &str, url: &str) -> String {
    let what = match kind {
        EventKind::PrNewComments => "new review comments",
        EventKind::CiFailed => "CI failing",
        EventKind::CiPassed => "CI passed",
    };
    format!("{what} on {ext} ({url})")
}

/// The PR with the user as its reviewer when they review their own PRs in that repo.
fn as_reviewed(pr: &fetch::Pr, solo: bool) -> fetch::Pr {
    if solo {
        fetch::Pr {
            reviewers: fetch::Reviewers::Yourself,
            ..pr.clone()
        }
    } else {
        pr.clone()
    }
}

/// The PR's CI state without its informational checks, whose failures become notes.
fn with_checks(pr: &fetch::Pr, informational: &[String]) -> fetch::Pr {
    if informational.is_empty() {
        return pr.clone();
    }
    fetch::Pr {
        ci: checks::rollup(&pr.checks, informational).or_else(|| pr.ci.clone()),
        notes: checks::notes(&pr.checks, informational),
        ..pr.clone()
    }
}

/// Pulls open PRs into items and returns the events they fired.
pub async fn sync(handle: &AppHandle) -> AppResult<Vec<Fired>> {
    let app = handle.state::<App>();
    let all = app.settings()?;
    let settings = all.github.clone();
    let prs = fetch::open_prs().await?;
    let now = Utc::now();
    app.db.with(|conn| {
        let mut fired = Vec::new();
        let mut open = HashSet::new();
        for pr in prs.iter().filter(|p| !settings.ignores(&p.repo)) {
            let pr = &as_reviewed(pr, settings.self_reviews(&pr.repo));
            let pr = &with_checks(pr, &settings.informational(&pr.repo));
            let pr = &with_checks(pr, &settings.informational(&pr.repo));
            let area = settings.area_for(&pr.repo);
            // Another machine syncs the areas it runs, so two never duplicate a PR.
            if all.node_for(area) != all.node {
                continue;
            }
            let ext = apply::external_id(pr);
            let kinds = apply::apply(conn, pr, area, now)?;
            let item_id = items::by_external(conn, SOURCE, &ext)?
                .map(|i| i.id)
                .unwrap_or_default();
            for kind in kinds {
                fired.push(Fired {
                    kind,
                    item_id: item_id.clone(),
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
