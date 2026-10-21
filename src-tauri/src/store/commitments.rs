use rusqlite::{Connection, OptionalExtension as _, Row, params};

use crate::domain::commitment::Commitment;
use crate::error::{AppError, AppResult};

const COLUMNS: &str = "id, title, area, starts_at, ends_at, location, source, external_id, hidden";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Commitment> {
    Ok(Commitment {
        id: row.get(0)?,
        title: row.get(1)?,
        area: row.get(2)?,
        starts_at: row.get(3)?,
        ends_at: row.get(4)?,
        location: row.get(5)?,
        source: row.get(6)?,
        external_id: row.get(7)?,
        hidden: row.get(8)?,
    })
}

fn query(conn: &Connection, sql: &str, args: impl rusqlite::Params) -> AppResult<Vec<Commitment>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(args, from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Commitments that are not hidden.
pub fn list(conn: &Connection) -> AppResult<Vec<Commitment>> {
    query(
        conn,
        &format!("SELECT {COLUMNS} FROM commitments WHERE hidden = 0"),
        [],
    )
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Commitment> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM commitments WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("meeting {id}")))
}

/// Stores a new commitment; its `id` is ignored and assigned by the database.
pub fn insert(conn: &Connection, c: &Commitment) -> AppResult<()> {
    conn.execute(
        "INSERT INTO commitments (title, area, starts_at, ends_at, location, source, external_id, hidden)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![c.title, c.area, c.starts_at, c.ends_at, c.location, c.source, c.external_id, c.hidden],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, c: &Commitment) -> AppResult<()> {
    conn.execute(
        "UPDATE commitments SET title = ?1, area = ?2, starts_at = ?3, ends_at = ?4, location = ?5,
         hidden = ?6 WHERE id = ?7",
        params![
            c.title,
            c.area,
            c.starts_at,
            c.ends_at,
            c.location,
            c.hidden,
            c.id
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM commitments WHERE id = ?1", [id])?;
    Ok(())
}
