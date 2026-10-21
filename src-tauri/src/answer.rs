//! The user answering an ask, or telling Claude what to do about an item. Either way it stays on
//! the item's history and waits for the scheduler to take it to Claude, but the item's state
//! changes at once: Claude has it if a session will get the answer, otherwise the answer stands.

use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

use crate::config::settings::Settings;
use crate::domain::ask::Ask;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Status, WorkItem};
use crate::error::{AppError, AppResult};
use crate::store::{asks, events, items, watch};
use crate::{work, workspace};

fn nonempty(text: &str) -> AppResult<&str> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::BadInput("that can't be empty".to_owned()));
    }
    Ok(text)
}

/// What came of a reply. A screen can show a question after it was answered or dismissed
/// elsewhere, so answering one that's gone is a normal outcome, not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Done,
    /// The question was already answered or dismissed; nothing changed.
    Gone,
}

pub fn give(
    conn: &Connection,
    settings: &Settings,
    ask_id: i64,
    answer: &str,
) -> AppResult<Outcome> {
    let answer = nonempty(answer)?;
    let Some(ask) = asks::find_open(conn, ask_id)? else {
        return Ok(Outcome::Gone);
    };
    let text = format!("You answered “{answer}” to: {}", ask.question);
    events::record(conn, &ask.item_id, Utc::now(), &Change::Note { text })?;
    asks::answer(conn, ask_id, answer)?;
    hand_over(conn, settings, &ask, answer)?;
    Ok(Outcome::Done)
}

/// A message about an item with no question behind it. Stored as an already-answered ask with no
/// question, so it reaches Claude the same way an answer does.
pub fn tell(conn: &Connection, settings: &Settings, item_id: &str, message: &str) -> AppResult<()> {
    let message = nonempty(message)?;
    items::get(conn, item_id)?;
    let ask = Ask {
        item_id: item_id.to_owned(),
        ..Ask::default()
    };
    let id = asks::insert(conn, &ask)?;
    let text = format!("You told Claude: {message}");
    events::record(conn, item_id, Utc::now(), &Change::Note { text })?;
    asks::answer(conn, id, message)?;
    hand_over(conn, settings, &ask, message)
}

/// Whether the scheduler will find a Claude session to take the answer: the one that asked, the
/// one holding the item, or a new one in the repo's project. Another machine's items are its own
/// to deliver.
fn reaches_claude(
    conn: &Connection,
    settings: &Settings,
    ask: &Ask,
    item: &WorkItem,
) -> AppResult<bool> {
    if settings.node_for(&item.area) != settings.node || ask.asker.is_some() {
        return Ok(true);
    }
    if watch::all(conn)?
        .get(&item.id)
        .is_some_and(|w| w.session.is_some())
    {
        return Ok(true);
    }
    Ok(workspace::for_item(settings, item)?.is_some())
}

/// Once nothing else on the item waits on the user, it stops showing as their decision.
fn hand_over(conn: &Connection, settings: &Settings, ask: &Ask, answer: &str) -> AppResult<()> {
    let item = items::get(conn, &ask.item_id)?;
    let waiting = !asks::for_item(conn, &item.id)?.is_empty();
    if waiting || item.status != Some(Status::Decision) {
        return Ok(());
    }
    let (status, holder, label) = if reaches_claude(conn, settings, ask, &item)? {
        (
            Some(Status::Proceeding),
            "Claude".to_owned(),
            "acting on your answer".to_owned(),
        )
    } else {
        (None, item.holder.clone(), format!("you answered: {answer}"))
    };
    let input = ItemInput {
        id: item.id,
        title: item.title,
        state_label: label,
        stage: item.stage,
        status,
        holder,
        area: item.area,
        link: item.link,
    };
    work::save(conn, input, Utc::now()).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::Stage;
    use crate::domain::session::SessionRef;
    use crate::store::schema;

    fn decision(conn: &Connection, id: &str) {
        let input = ItemInput {
            id: id.to_owned(),
            title: "t".to_owned(),
            state_label: "your call".to_owned(),
            stage: Stage::InReview,
            status: Some(Status::Decision),
            holder: "you".to_owned(),
            area: "work".to_owned(),
            link: None,
        };
        work::save(conn, input, Utc::now()).unwrap();
    }

    fn ask(conn: &Connection, item: &str, asker: Option<SessionRef>) -> i64 {
        let ask = Ask {
            item_id: item.to_owned(),
            question: "Merge?".to_owned(),
            asker,
            ..Ask::default()
        };
        asks::insert(conn, &ask).unwrap()
    }

    #[test]
    fn answering_updates_the_item_at_once() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        let settings = Settings::default();
        decision(&conn, "a");
        let session = SessionRef {
            id: "s".to_owned(),
            cwd: None,
        };
        let first = ask(&conn, "a", Some(session));
        give(&conn, &settings, first, "Yes").unwrap();
        let a = items::get(&conn, "a").unwrap();
        assert_eq!(
            (a.status, a.holder.as_str()),
            (Some(Status::Proceeding), "Claude")
        );

        decision(&conn, "b");
        let second = ask(&conn, "b", None);
        give(&conn, &settings, second, "No").unwrap();
        let b = items::get(&conn, "b").unwrap();
        assert_eq!(
            (b.status, b.state_label.as_str()),
            (None, "you answered: No")
        );
    }

    #[test]
    fn answering_a_question_that_is_gone_changes_nothing() {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        let settings = Settings::default();
        decision(&conn, "a");
        let answered = ask(&conn, "a", None);
        let dismissed = ask(&conn, "a", None);
        assert_eq!(
            give(&conn, &settings, answered, "Yes").unwrap(),
            Outcome::Done
        );
        asks::remove(&conn, dismissed).unwrap();
        let history = events::for_item(&conn, "a").unwrap().len();
        assert_eq!(
            give(&conn, &settings, answered, "No").unwrap(),
            Outcome::Gone
        );
        assert_eq!(
            give(&conn, &settings, dismissed, "No").unwrap(),
            Outcome::Gone
        );
        assert_eq!(events::for_item(&conn, "a").unwrap().len(), history);
    }
}
