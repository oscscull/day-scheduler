//! Items Claude works on one at a time, in the order `[queue]` lists them. The first one not yet
//! in review or done is current: it starts when it's up next, and the queue waits while it's in
//! progress. Starting one hands Claude `prompts/queue.md` the way `tell_claude` hands it a message.

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::domain::ask::Ask;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Stage, Status, WorkItem};
use crate::error::AppResult;
use crate::store::{asks, events};
use crate::work;

#[derive(Debug, Clone, Copy)]
pub enum Next<'a> {
    /// Up next: start it.
    Start(&'a WorkItem),
    /// Already in progress: wait for it.
    Wait(&'a WorkItem),
    /// Everything listed is in review or done, or gone.
    Finished,
}

pub fn next<'a>(queue: &[String], items: &'a [WorkItem]) -> Next<'a> {
    for id in queue {
        let Some(item) = items.iter().find(|i| &i.id == id) else {
            continue;
        };
        match item.stage {
            Stage::InReview | Stage::Done => {}
            Stage::UpNext => return Next::Start(item),
            Stage::InProgress => return Next::Wait(item),
        }
    }
    Next::Finished
}

/// Leaves `message` for the scheduler to take to Claude, and marks the item in progress so the
/// queue waits for it.
pub fn start(
    conn: &Connection,
    item: &WorkItem,
    message: &str,
    now: DateTime<Utc>,
) -> AppResult<()> {
    let ask = Ask {
        item_id: item.id.clone(),
        ..Ask::default()
    };
    let id = asks::insert(conn, &ask)?;
    asks::answer(conn, id, message)?;
    let text = "Started from the queue".to_owned();
    events::record(conn, &item.id, now, &Change::Note { text })?;
    let input = ItemInput {
        id: item.id.clone(),
        title: item.title.clone(),
        state_label: "Claude is on it".to_owned(),
        stage: Stage::InProgress,
        status: Some(Status::Proceeding),
        holder: "Claude".to_owned(),
        area: item.area.clone(),
        link: item.link.clone(),
    };
    work::save(conn, input, now).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{items, schema};

    fn item(id: &str, stage: Stage) -> WorkItem {
        WorkItem {
            id: id.to_owned(),
            title: "t".to_owned(),
            state_label: String::new(),
            stage,
            status: None,
            holder: String::new(),
            area: "projects".to_owned(),
            link: None,
            source: "manual".to_owned(),
            external_id: None,
            opened_at: Utc::now(),
            state_since: Utc::now(),
            rank: 0,
        }
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn works_through_items_one_at_a_time_in_order() {
        let all = [
            item("a", Stage::InReview),
            item("b", Stage::UpNext),
            item("c", Stage::UpNext),
            item("d", Stage::InProgress),
        ];
        assert!(
            matches!(next(&ids(&["a", "gone", "b", "c"]), &all), Next::Start(i) if i.id == "b")
        );
        assert!(matches!(next(&ids(&["d", "b"]), &all), Next::Wait(i) if i.id == "d"));
        assert!(matches!(next(&ids(&["a"]), &all), Next::Finished));
        assert!(matches!(next(&[], &all), Next::Finished));
    }

    #[test]
    fn starting_leaves_a_message_and_holds_the_queue() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        let up = item("b", Stage::UpNext);
        items::save(&conn, &up).unwrap();
        start(&conn, &up, "Fix it.", Utc::now()).unwrap();
        let waiting = asks::answered(&conn).unwrap();
        assert_eq!(waiting.first().unwrap().answer, "Fix it.");
        let all = items::list(&conn).unwrap();
        assert!(matches!(next(&ids(&["b"]), &all), Next::Wait(_)));
    }
}
