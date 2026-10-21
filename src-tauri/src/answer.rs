//! The user answering an ask. The answer stays on the item's history, and waits for the scheduler
//! to take it back to whoever asked.

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::domain::event::Change;
use crate::error::{AppError, AppResult};
use crate::store::{asks, events};

pub fn give(conn: &Connection, ask_id: i64, answer: &str, now: DateTime<Utc>) -> AppResult<()> {
    let answer = answer.trim();
    if answer.is_empty() {
        return Err(AppError::BadInput("an answer can't be empty".to_owned()));
    }
    let ask = asks::get(conn, ask_id)?;
    let text = format!("You answered “{answer}” to: {}", ask.question);
    events::record(conn, &ask.item_id, now, &Change::Note { text })?;
    asks::answer(conn, ask_id, answer)
}
