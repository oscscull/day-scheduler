use rusqlite::{Connection, Row, params};

use super::codec::{column, encode, required};
use crate::domain::item::WorkItem;
use crate::error::AppResult;

const SELECT: &str = "SELECT id, title, state_label, stage, status, holder, area, opened_at, rank
                      FROM items ORDER BY rank";

fn from_row(row: &Row<'_>) -> rusqlite::Result<WorkItem> {
    Ok(WorkItem {
        id: row.get(0)?,
        title: row.get(1)?,
        state_label: row.get(2)?,
        stage: required(row, 3)?,
        status: column(row, 4)?,
        holder: row.get(5)?,
        area: row.get(6)?,
        opened_at: row.get(7)?,
        rank: row.get(8)?,
    })
}

/// All items, highest priority first.
pub fn list(conn: &Connection) -> AppResult<Vec<WorkItem>> {
    let mut stmt = conn.prepare(SELECT)?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn insert(conn: &Connection, item: &WorkItem) -> AppResult<()> {
    conn.execute(
        "INSERT INTO items (id, title, state_label, stage, status, holder, area, opened_at, rank)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            item.id,
            item.title,
            item.state_label,
            encode(&item.stage)?,
            item.status.map(|s| encode(&s)).transpose()?,
            item.holder,
            item.area,
            item.opened_at,
            item.rank
        ],
    )?;
    Ok(())
}

/// Sets each item's rank to its position in `ids`.
pub fn reorder(conn: &Connection, ids: &[String]) -> AppResult<()> {
    let mut stmt = conn.prepare("UPDATE items SET rank = ?1 WHERE id = ?2")?;
    for (rank, id) in (0_i64..).zip(ids) {
        stmt.execute(params![rank, id])?;
    }
    Ok(())
}
