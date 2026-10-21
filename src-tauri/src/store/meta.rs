//! Small key-value facts about the app itself, e.g. when the scheduler last ran.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension as _, params};

use crate::error::AppResult;

pub fn time(conn: &Connection, key: &str) -> AppResult<Option<DateTime<Utc>>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

pub fn set_time(conn: &Connection, key: &str, at: DateTime<Utc>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, at],
    )?;
    Ok(())
}

pub fn text(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
        .optional()?)
}

pub fn set_text(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
