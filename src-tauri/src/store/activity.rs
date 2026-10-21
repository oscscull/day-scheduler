use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, params};

use crate::domain::activity::Activity;
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id: row.get(0)?,
        text: row.get(1)?,
        at: row.get(2)?,
        unread: row.get(3)?,
    })
}

/// The most recent `limit` entries, newest first.
pub fn recent(conn: &Connection, limit: u32) -> AppResult<Vec<Activity>> {
    let mut stmt =
        conn.prepare("SELECT id, text, at, unread FROM activity ORDER BY at DESC LIMIT ?1")?;
    let rows = stmt.query_map([limit], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn insert(conn: &Connection, text: &str, at: DateTime<Utc>, unread: bool) -> AppResult<()> {
    conn.execute(
        "INSERT INTO activity (text, at, unread) VALUES (?1, ?2, ?3)",
        params![text, at, unread],
    )?;
    Ok(())
}
