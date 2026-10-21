//! Deletes stored agent output once it has outlived its TTL.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

use super::items;
use crate::error::AppResult;

/// How long each kind of stored output lives.
#[derive(Debug, Clone, Copy)]
pub struct Ttl {
    pub activity: Duration,
    pub runs: Duration,
    pub done_todos: Duration,
    pub done_items: Duration,
}

/// Removes stored output past its TTL: activity, finished runs, done to-dos, and done items with
/// their history. Live runs are kept.
pub fn expire(conn: &Connection, now: DateTime<Utc>, ttl: Ttl) -> AppResult<()> {
    conn.execute(
        "DELETE FROM activity WHERE at < ?1",
        params![now - ttl.activity],
    )?;
    conn.execute(
        "DELETE FROM runs WHERE finished_at IS NOT NULL AND finished_at < ?1",
        params![now - ttl.runs],
    )?;
    conn.execute(
        "DELETE FROM todos WHERE done_at IS NOT NULL AND done_at < ?1",
        params![now - ttl.done_todos],
    )?;
    for id in items::done_before(conn, now - ttl.done_items)? {
        items::delete(conn, &id)?;
    }
    Ok(())
}
