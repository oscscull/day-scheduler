use rusqlite::{Connection, Row, params};

use crate::domain::activity::Activity;
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id: row.get(0)?,
        text: row.get(1)?,
        area: row.get(2)?,
        at: row.get(3)?,
        unread: row.get(4)?,
    })
}

/// The most recent `limit` entries, newest first.
pub fn recent(conn: &Connection, limit: u32) -> AppResult<Vec<Activity>> {
    let mut stmt =
        conn.prepare("SELECT id, text, area, at, unread FROM activity ORDER BY at DESC LIMIT ?1")?;
    let rows = stmt.query_map([limit], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Stores a note; its `id` is ignored and assigned by the database.
pub fn insert(conn: &Connection, a: &Activity) -> AppResult<()> {
    conn.execute(
        "INSERT INTO activity (text, area, at, unread) VALUES (?1, ?2, ?3, ?4)",
        params![a.text, a.area, a.at, a.unread],
    )?;
    Ok(())
}
