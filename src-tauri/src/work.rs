//! Creating and editing work items, recording each change in the item's history.

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::domain::event::Change;
use crate::domain::item::{ItemInput, WorkItem};
use crate::error::AppResult;
use crate::store::{events, items};

/// Saves an item from the UI or an agent and returns its id. Unknown or empty ids create one.
pub fn save(conn: &Connection, input: ItemInput, now: DateTime<Utc>) -> AppResult<String> {
    let existing = if input.id.is_empty() {
        None
    } else {
        items::find(conn, &input.id)?
    };
    match existing {
        Some(old) => update(conn, &old, input, now),
        None => create(conn, input, now),
    }
}

fn create(conn: &Connection, input: ItemInput, now: DateTime<Utc>) -> AppResult<String> {
    let id = if input.id.trim().is_empty() {
        fresh_id(conn)?
    } else {
        input.id.trim().to_owned()
    };
    let item = WorkItem {
        id: id.clone(),
        opened_at: now,
        state_since: now,
        rank: items::next_rank(conn)?,
        ..from_input(input)
    };
    items::save(conn, &item)?;
    events::record(conn, &id, now, &Change::Created)?;
    Ok(id)
}

fn update(
    conn: &Connection,
    old: &WorkItem,
    input: ItemInput,
    now: DateTime<Utc>,
) -> AppResult<String> {
    let mut item = WorkItem {
        id: old.id.clone(),
        opened_at: old.opened_at,
        state_since: old.state_since,
        rank: old.rank,
        source: old.source.clone(),
        external_id: old.external_id.clone(),
        ..from_input(input)
    };
    let changes = diff(old, &item);
    if !changes.is_empty() {
        item.state_since = now;
    }
    items::save(conn, &item)?;
    for change in &changes {
        events::record(conn, &item.id, now, change)?;
    }
    Ok(item.id)
}

/// The fields filled from input; ids, times and rank are set by the caller.
fn from_input(input: ItemInput) -> WorkItem {
    WorkItem {
        id: String::new(),
        title: input.title,
        state_label: input.state_label,
        stage: input.stage,
        status: input.status,
        holder: input.holder,
        area: input.area,
        link: input.link.filter(|l| !l.trim().is_empty()),
        source: items::MANUAL.to_owned(),
        external_id: None,
        opened_at: DateTime::<Utc>::MIN_UTC,
        state_since: DateTime::<Utc>::MIN_UTC,
        rank: 0,
    }
}

/// Changes worth a line in the history. Title, holder and link edits are housekeeping.
fn diff(old: &WorkItem, new: &WorkItem) -> Vec<Change> {
    let mut changes = Vec::new();
    if old.stage != new.stage {
        changes.push(Change::Stage { stage: new.stage });
    }
    if old.status != new.status {
        changes.push(Change::Status { status: new.status });
    }
    if old.state_label != new.state_label {
        changes.push(Change::Label {
            label: new.state_label.clone(),
        });
    }
    changes
}

/// "B-1", "B-2"… for items made without a ticket key.
fn fresh_id(conn: &Connection) -> AppResult<String> {
    let mut n = conn.query_row("SELECT COUNT(*) + 1 FROM items", [], |r| r.get::<_, i64>(0))?;
    while items::find(conn, &format!("B-{n}"))?.is_some() {
        n += 1;
    }
    Ok(format!("B-{n}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::{Stage, Status};
    use crate::store::schema;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        conn
    }

    fn input(id: &str, stage: Stage) -> ItemInput {
        ItemInput {
            id: id.to_owned(),
            title: "Thing".to_owned(),
            state_label: "starting".to_owned(),
            stage,
            status: Some(Status::Proceeding),
            holder: "Claude".to_owned(),
            area: "work".to_owned(),
            link: Some(String::new()),
        }
    }

    #[test]
    fn creates_with_generated_ids_at_the_bottom() {
        let conn = db();
        let now = Utc::now();
        assert_eq!(save(&conn, input("", Stage::UpNext), now).unwrap(), "B-1");
        assert_eq!(
            save(&conn, input("JD-1", Stage::UpNext), now).unwrap(),
            "JD-1"
        );
        let ranks: Vec<i64> = items::list(&conn).unwrap().iter().map(|i| i.rank).collect();
        assert_eq!(ranks, [0, 1]);
        assert_eq!(items::get(&conn, "B-1").unwrap().link, None);
    }

    #[test]
    fn records_stage_changes_only_when_they_happen() {
        let conn = db();
        let id = save(&conn, input("", Stage::UpNext), Utc::now()).unwrap();
        save(&conn, input(&id, Stage::UpNext), Utc::now()).unwrap();
        save(&conn, input(&id, Stage::InReview), Utc::now()).unwrap();
        let history: Vec<Change> = events::for_item(&conn, &id)
            .unwrap()
            .into_iter()
            .map(|e| e.change)
            .collect();
        assert_eq!(
            history,
            [
                Change::Stage {
                    stage: Stage::InReview
                },
                Change::Created
            ]
        );
    }
}
