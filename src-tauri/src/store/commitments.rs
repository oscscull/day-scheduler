use rusqlite::{Connection, Row, params};

use crate::domain::commitment::Commitment;
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Commitment> {
    Ok(Commitment {
        id: row.get(0)?,
        title: row.get(1)?,
        area: row.get(2)?,
        starts_at: row.get(3)?,
        ends_at: row.get(4)?,
        location: row.get(5)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Commitment>> {
    let mut stmt =
        conn.prepare("SELECT id, title, area, starts_at, ends_at, location FROM commitments")?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn insert(conn: &Connection, c: &Commitment) -> AppResult<()> {
    conn.execute(
        "INSERT INTO commitments (title, area, starts_at, ends_at, location)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![c.title, c.area, c.starts_at, c.ends_at, c.location],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM commitments WHERE id = ?1", [id])?;
    Ok(())
}
