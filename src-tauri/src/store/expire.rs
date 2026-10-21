//! Deletes stored agent output once it has outlived its TTL.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

use crate::error::AppResult;

/// How long each kind of stored output lives.
#[derive(Debug, Clone, Copy)]
pub struct Ttl {
    pub activity: Duration,
    pub runs: Duration,
    pub done_todos: Duration,
}

/// Removes activity, finished runs and done to-dos older than their TTL. Live runs are kept.
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
    Ok(())
}
