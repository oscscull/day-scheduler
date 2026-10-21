//! Deletes stored agent output once it has outlived its TTL.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

use crate::error::AppResult;

/// How long each kind of stored output lives.
#[derive(Debug, Clone, Copy)]
pub struct Ttl {
    pub activity: Duration,
    pub runs: Duration,
}

/// Removes activity and finished runs older than their TTL. Live runs are kept.
pub fn expire(conn: &Connection, now: DateTime<Utc>, ttl: Ttl) -> AppResult<()> {
    conn.execute(
        "DELETE FROM activity WHERE at < ?1",
        params![now - ttl.activity],
    )?;
    conn.execute(
        "DELETE FROM runs WHERE finished_at IS NOT NULL AND finished_at < ?1",
        params![now - ttl.runs],
    )?;
    Ok(())
}
