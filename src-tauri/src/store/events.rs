use chrono::{DateTime, Utc};
use rusqlite::types::Type;
use rusqlite::{Connection, Row, params};

use crate::domain::event::{Change, ItemEvent};
use crate::error::AppResult;

fn from_row(row: &Row<'_>) -> rusqlite::Result<ItemEvent> {
    let json: String = row.get(3)?;
    let change = serde_json::from_str(&json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?;
    Ok(ItemEvent {
        id: row.get(0)?,
        item_id: row.get(1)?,
        at: row.get(2)?,
        change,
    })
}

/// An item's history, newest first.
pub fn for_item(conn: &Connection, item_id: &str) -> AppResult<Vec<ItemEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, item_id, at, change FROM item_events WHERE item_id = ?1 ORDER BY at DESC, id DESC",
    )?;
    let rows = stmt.query_map([item_id], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn record(
    conn: &Connection,
    item_id: &str,
    at: DateTime<Utc>,
    change: &Change,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO item_events (item_id, at, change) VALUES (?1, ?2, ?3)",
        params![item_id, at, serde_json::to_string(change)?],
    )?;
    Ok(())
}
