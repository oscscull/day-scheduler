//! An item whose PR Meatpuppet opened (an issue it fixed) becomes that PR's item, so syncs update
//! it rather than adding a second one.

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use super::apply::SOURCE;
use crate::domain::activity::Activity;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Stage, Status, WorkItem};
use crate::error::AppResult;
use crate::store::{activity, events, items};
use crate::work;

/// The PR that the item's work became.
#[derive(Debug, Clone, Copy)]
pub struct OpenedPr<'a> {
    /// `owner/repo`.
    pub repo: &'a str,
    pub number: u64,
    pub url: &'a str,
}

pub fn adopt(
    conn: &Connection,
    item_id: &str,
    pr: OpenedPr<'_>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let ext = format!("{}#{}", pr.repo, pr.number);
    // A sync between opening the PR and now made an item of its own for it.
    if let Some(twin) = items::by_external(conn, SOURCE, &ext)?
        && twin.id != item_id
    {
        items::purge(conn, &twin.id)?;
    }
    let item = items::get(conn, item_id)?;
    let input = ItemInput {
        id: item.id.clone(),
        title: item.title.clone(),
        state_label: format!("PR #{} open", pr.number),
        stage: Stage::InReview,
        status: Some(Status::Ci),
        holder: item.holder.clone(),
        area: item.area.clone(),
        link: Some(pr.url.to_owned()),
    };
    work::save(conn, input, now)?;
    let synced = WorkItem {
        source: SOURCE.to_owned(),
        external_id: Some(ext),
        ..items::get(conn, item_id)?
    };
    items::save(conn, &synced)?;
    let text = format!("Meatpuppet opened {}", pr.url);
    events::record(conn, item_id, now, &Change::Note { text })?;
    let update = Activity {
        id: 0,
        text: format!("Opened PR #{} for {item_id}: {}", pr.number, item.title),
        area: item.area,
        at: now,
        unread: true,
    };
    activity::insert(conn, &update)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::schema;

    fn save(conn: &Connection, id: &str, link: &str) {
        let input = ItemInput {
            id: id.to_owned(),
            title: "Upsert breaks".to_owned(),
            state_label: "up next".to_owned(),
            stage: Stage::UpNext,
            status: None,
            holder: "Claude".to_owned(),
            area: "projects".to_owned(),
            link: Some(link.to_owned()),
        };
        work::save(conn, input, Utc::now()).unwrap();
    }

    #[test]
    fn the_issue_item_becomes_the_prs_and_its_twin_goes() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        save(
            &conn,
            "baseddsl#178",
            "https://github.com/o/baseddsl/issues/178",
        );
        save(
            &conn,
            "baseddsl#190",
            "https://github.com/o/baseddsl/pull/190",
        );
        let twin = WorkItem {
            source: SOURCE.to_owned(),
            external_id: Some("o/baseddsl#190".to_owned()),
            ..items::get(&conn, "baseddsl#190").unwrap()
        };
        items::save(&conn, &twin).unwrap();

        let pr = OpenedPr {
            repo: "o/baseddsl",
            number: 190,
            url: "https://github.com/o/baseddsl/pull/190",
        };
        adopt(&conn, "baseddsl#178", pr, Utc::now()).unwrap();

        let item = items::by_external(&conn, SOURCE, "o/baseddsl#190")
            .unwrap()
            .unwrap();
        assert_eq!(item.id, "baseddsl#178");
        assert_eq!(item.stage, Stage::InReview);
        assert_eq!(item.link.as_deref(), Some(pr.url));
        assert!(items::find(&conn, "baseddsl#190").unwrap().is_none());
    }
}
