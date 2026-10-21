//! The user answering an ask, or telling Claude what to do about an item. Either way it stays on
//! the item's history, and waits for the scheduler to take it to Claude.

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::domain::ask::Ask;
use crate::domain::event::Change;
use crate::error::{AppError, AppResult};
use crate::store::{asks, events, items};

fn nonempty(text: &str) -> AppResult<&str> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::BadInput("that can't be empty".to_owned()));
    }
    Ok(text)
}

pub fn give(conn: &Connection, ask_id: i64, answer: &str, now: DateTime<Utc>) -> AppResult<()> {
    let answer = nonempty(answer)?;
    let ask = asks::get(conn, ask_id)?;
    let text = format!("You answered “{answer}” to: {}", ask.question);
    events::record(conn, &ask.item_id, now, &Change::Note { text })?;
    asks::answer(conn, ask_id, answer)
}

/// A message about an item with no question behind it. Stored as an already-answered ask with no
/// question, so it reaches Claude the same way an answer does.
pub fn tell(conn: &Connection, item_id: &str, message: &str, now: DateTime<Utc>) -> AppResult<()> {
    let message = nonempty(message)?;
    items::get(conn, item_id)?;
    let ask = Ask {
        item_id: item_id.to_owned(),
        ..Ask::default()
    };
    let id = asks::insert(conn, &ask)?;
    let text = format!("You told Claude: {message}");
    events::record(conn, item_id, now, &Change::Note { text })?;
    asks::answer(conn, id, message)
}
