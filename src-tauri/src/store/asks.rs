use rusqlite::{Connection, Row, params};

use crate::domain::ask::Ask;
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Ask> {
    Ok(Ask {
        id: row.get(0)?,
        item_id: row.get(1)?,
        question: row.get(2)?,
        context: row.get(3)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Ask>> {
    let mut stmt = conn.prepare("SELECT id, item_id, question, context FROM asks ORDER BY id")?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn insert(conn: &Connection, item_id: &str, question: &str, context: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO asks (item_id, question, context) VALUES (?1, ?2, ?3)",
        params![item_id, question, context],
    )?;
    Ok(())
}
