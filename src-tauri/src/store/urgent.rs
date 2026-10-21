use rusqlite::{Connection, OptionalExtension as _, Row, params};

use crate::domain::urgent::Urgent;
use crate::error::{AppError, AppResult};

const COLUMNS: &str = "id, title, detail, raised_at, schedule_id";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Urgent> {
    Ok(Urgent {
        id: row.get(0)?,
        title: row.get(1)?,
        detail: row.get(2)?,
        raised_at: row.get(3)?,
        schedule_id: row.get(4)?,
    })
}

/// Open urgent items, newest first.
pub fn list(conn: &Connection) -> AppResult<Vec<Urgent>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM urgent ORDER BY raised_at DESC"
    ))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn find(conn: &Connection, id: i64) -> AppResult<Urgent> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM urgent WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("urgent item {id}")))
}

/// Stores a new urgent item; its `id` is ignored and assigned by the database.
pub fn raise(conn: &Connection, item: &Urgent) -> AppResult<()> {
    conn.execute(
        "INSERT INTO urgent (title, detail, raised_at, schedule_id) VALUES (?1, ?2, ?3, ?4)",
        params![item.title, item.detail, item.raised_at, item.schedule_id],
    )?;
    Ok(())
}

/// Dismissing deletes: an urgent item has no life after it is dealt with.
pub fn dismiss(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM urgent WHERE id = ?1", [id])?;
    Ok(())
}

pub fn dismiss_all(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM urgent", [])?;
    Ok(())
}
