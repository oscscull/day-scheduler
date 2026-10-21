use rusqlite::{Connection, Row, params};

use crate::domain::commitment::Commitment;
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Commitment> {
    Ok(Commitment {
        id: row.get(0)?,
        title: row.get(1)?,
        starts_at: row.get(2)?,
        ends_at: row.get(3)?,
        location: row.get(4)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Commitment>> {
    let mut stmt =
        conn.prepare("SELECT id, title, starts_at, ends_at, location FROM commitments")?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn insert(conn: &Connection, c: &Commitment) -> AppResult<()> {
    conn.execute(
        "INSERT INTO commitments (title, starts_at, ends_at, location) VALUES (?1, ?2, ?3, ?4)",
        params![c.title, c.starts_at, c.ends_at, c.location],
    )?;
    Ok(())
}
