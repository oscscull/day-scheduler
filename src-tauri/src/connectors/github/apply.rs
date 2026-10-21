//! Writes PRs into Bridge as items, respecting edits made since the last sync.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use super::fetch::Pr;
use super::snapshot::{self, Snapshot};
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Stage, WorkItem};
use crate::domain::schedule::EventKind;
use crate::error::AppResult;
use crate::store::{events, items, sync_state};
use crate::work;

pub const SOURCE: &str = "github";

pub fn external_id(pr: &Pr) -> String {
    format!("{}#{}", pr.repo, pr.number)
}

/// A short display id, e.g. `app#12`.
fn display_id(pr: &Pr) -> String {
    let name = pr
        .repo
        .rsplit_once('/')
        .map_or(pr.repo.as_str(), |(_, n)| n);
    format!("{name}#{}", pr.number)
}

fn input(id: String, s: &Snapshot, area: String, link: Option<String>) -> ItemInput {
    ItemInput {
        id,
        title: s.owned.title.clone(),
        state_label: s.owned.label.clone(),
        stage: s.owned.stage,
        status: s.owned.status,
        holder: s.owned.holder.clone(),
        area,
        link,
    }
}

/// Creates or updates the item for one PR and returns the events it fired.
pub fn apply(
    conn: &Connection,
    pr: &Pr,
    area: &str,
    now: DateTime<Utc>,
) -> AppResult<Vec<EventKind>> {
    let ext = external_id(pr);
    let new = snapshot::of(pr);
    let old: Option<Snapshot> =
        sync_state::get(conn, SOURCE, &ext)?.and_then(|json| serde_json::from_str(&json).ok());
    match items::by_external(conn, SOURCE, &ext)? {
        None => create(conn, pr, area, now)?,
        Some(item) if old.as_ref().map(|o| &o.owned) != Some(&new.owned) => {
            work::save(conn, input(item.id, &new, item.area, item.link), now)?;
        }
        Some(_) => {}
    }
    note_changes(conn, &ext, (old.as_ref(), &new), now)?;
    sync_state::set(conn, SOURCE, &ext, &serde_json::to_string(&new)?)?;
    Ok(snapshot::events(old.as_ref(), &new))
}

/// Adds informational check results to the item's history the first time each is seen.
fn note_changes(
    conn: &Connection,
    ext: &str,
    (old, new): (Option<&Snapshot>, &Snapshot),
    now: DateTime<Utc>,
) -> AppResult<()> {
    let Some(item) = items::by_external(conn, SOURCE, ext)? else {
        return Ok(());
    };
    let seen = old.map(|o| o.notes.as_slice()).unwrap_or_default();
    for text in new.notes.iter().filter(|n| !seen.contains(n)) {
        events::record(conn, &item.id, now, &Change::Note { text: text.clone() })?;
    }
    Ok(())
}

fn create(conn: &Connection, pr: &Pr, area: &str, now: DateTime<Utc>) -> AppResult<()> {
    let s = &snapshot::of(pr);
    let id = work::save(
        conn,
        input(display_id(pr), s, area.to_owned(), Some(pr.url.clone())),
        now,
    )?;
    let item = WorkItem {
        source: SOURCE.to_owned(),
        external_id: Some(external_id(pr)),
        ..items::get(conn, &id)?
    };
    items::save(conn, &item)
}

/// PRs no longer open were merged or closed: mark their items done, once.
pub fn close_missing(
    conn: &Connection,
    open: &HashSet<String>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    for item in items::from_source(conn, SOURCE)? {
        let Some(ext) = item.external_id.clone() else {
            continue;
        };
        if open.contains(&ext) || item.stage == Stage::Done {
            continue;
        }
        let done = ItemInput {
            id: item.id,
            title: item.title,
            state_label: "merged or closed".to_owned(),
            stage: Stage::Done,
            status: None,
            holder: item.holder,
            area: item.area,
            link: item.link,
        };
        work::save(conn, done, now)?;
        sync_state::forget(conn, SOURCE, &ext)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::schema;

    fn pr(comments: u64) -> Pr {
        Pr {
            repo: "me/app".to_owned(),
            number: 7,
            title: "Fix login".to_owned(),
            url: "https://github.com/me/app/pull/7".to_owned(),
            draft: false,
            review: None,
            ci: Some("SUCCESS".to_owned()),
            comments,
            reviewers: super::super::fetch::Reviewers::Requested,
            reviewer_names: Vec::new(),
            checks: Vec::new(),
            notes: Vec::new(),
        }
    }

    #[test]
    fn syncs_keep_edits_until_github_changes() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        apply(&conn, &pr(0), "work", Utc::now()).unwrap();
        let mut edited = items::get(&conn, "app#7").unwrap();
        edited.state_label = "Sam is on holiday".to_owned();
        items::save(&conn, &edited).unwrap();
        let events = apply(&conn, &pr(1), "work", Utc::now()).unwrap();
        assert_eq!(events, [EventKind::PrNewComments]);
        assert_eq!(
            items::get(&conn, "app#7").unwrap().state_label,
            "Sam is on holiday"
        );
        close_missing(&conn, &HashSet::new(), Utc::now()).unwrap();
        assert_eq!(items::get(&conn, "app#7").unwrap().stage, Stage::Done);
    }
}
