use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension as _, Row, params};

use super::codec::{column, encode, required};
use crate::domain::item::WorkItem;
use crate::error::{AppError, AppResult};

const COLUMNS: &str =
    "id, title, state_label, stage, status, holder, area, link, opened_at, state_since, rank";

fn from_row(row: &Row<'_>) -> rusqlite::Result<WorkItem> {
    Ok(WorkItem {
        id: row.get(0)?,
        title: row.get(1)?,
        state_label: row.get(2)?,
        stage: required(row, 3)?,
        status: column(row, 4)?,
        holder: row.get(5)?,
        area: row.get(6)?,
        link: row.get(7)?,
        opened_at: row.get(8)?,
        state_since: row.get(9)?,
        rank: row.get(10)?,
    })
}

/// All items, highest priority first.
pub fn list(conn: &Connection) -> AppResult<Vec<WorkItem>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM items ORDER BY rank"))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn find(conn: &Connection, id: &str) -> AppResult<Option<WorkItem>> {
    let sql = format!("SELECT {COLUMNS} FROM items WHERE id = ?1");
    Ok(conn.query_row(&sql, [id], from_row).optional()?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<WorkItem> {
    find(conn, id)?.ok_or_else(|| AppError::NotFound(format!("item {id}")))
}

/// The rank that puts a new item at the bottom of the priorities.
pub fn next_rank(conn: &Connection) -> AppResult<i64> {
    Ok(
        conn.query_row("SELECT COALESCE(MAX(rank) + 1, 0) FROM items", [], |r| {
            r.get(0)
        })?,
    )
}

/// Inserts or replaces an item by id.
pub fn save(conn: &Connection, item: &WorkItem) -> AppResult<()> {
    conn.execute(
        &format!(
            "INSERT OR REPLACE INTO items ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        ),
        params![
            item.id,
            item.title,
            item.state_label,
            encode(&item.stage)?,
            item.status.map(|s| encode(&s)).transpose()?,
            item.holder,
            item.area,
            item.link,
            item.opened_at,
            item.state_since,
            item.rank
        ],
    )?;
    Ok(())
}

/// Deletes an item along with its asks and history.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM asks WHERE item_id = ?1", [id])?;
    conn.execute("DELETE FROM item_events WHERE item_id = ?1", [id])?;
    conn.execute("DELETE FROM items WHERE id = ?1", [id])?;
    Ok(())
}

/// Ids of done items whose state has not changed since `before`.
pub fn done_before(conn: &Connection, before: DateTime<Utc>) -> AppResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT id FROM items WHERE stage = 'done' AND state_since < ?1")?;
    let rows = stmt.query_map([before], |r| r.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Sets each item's rank to its position in `ids`.
pub fn reorder(conn: &Connection, ids: &[String]) -> AppResult<()> {
    let mut stmt = conn.prepare("UPDATE items SET rank = ?1 WHERE id = ?2")?;
    for (rank, id) in (0_i64..).zip(ids) {
        stmt.execute(params![rank, id])?;
    }
    Ok(())
}
