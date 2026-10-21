use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension as _, Row, params};

use super::codec::{column, encode, required};
use crate::domain::item::WorkItem;
use crate::error::{AppError, AppResult};

/// Items made by hand or by an agent, as opposed to a connector.
pub const MANUAL: &str = "manual";

const COLUMNS: &str = "id, title, state_label, stage, status, holder, area, link, source, external_id, opened_at, state_since, rank";

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
        source: row.get(8)?,
        external_id: row.get(9)?,
        opened_at: row.get(10)?,
        state_since: row.get(11)?,
        rank: row.get(12)?,
    })
}

/// Visible items, highest priority first.
pub fn list(conn: &Connection) -> AppResult<Vec<WorkItem>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM items WHERE hidden = 0 ORDER BY rank"
    ))?;
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
            "INSERT OR REPLACE INTO items ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
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
            item.source,
            item.external_id,
            item.opened_at,
            item.state_since,
            item.rank
        ],
    )?;
    Ok(())
}

/// Deletes an item along with its asks and history. Synced items are hidden instead, so the
/// next sync doesn't bring them back.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    if find(conn, id)?.is_some_and(|i| i.source != MANUAL) {
        return hide(conn, id);
    }
    purge(conn, id)
}

/// Deletes an item along with its asks and history, whatever made it.
pub fn purge(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM asks WHERE item_id = ?1", [id])?;
    conn.execute("DELETE FROM item_events WHERE item_id = ?1", [id])?;
    conn.execute("DELETE FROM watch WHERE item_id = ?1", [id])?;
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

/// The item a connector made for `external_id`, hidden or not.
pub fn by_external(
    conn: &Connection,
    source: &str,
    external_id: &str,
) -> AppResult<Option<WorkItem>> {
    let sql = format!("SELECT {COLUMNS} FROM items WHERE source = ?1 AND external_id = ?2");
    Ok(conn
        .query_row(&sql, [source, external_id], from_row)
        .optional()?)
}

/// Every item a connector made, hidden or not.
pub fn from_source(conn: &Connection, source: &str) -> AppResult<Vec<WorkItem>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM items WHERE source = ?1"))?;
    let rows = stmt.query_map([source], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Hides a synced item so the connector doesn't bring it back.
pub fn hide(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("UPDATE items SET hidden = 1 WHERE id = ?1", [id])?;
    Ok(())
}
