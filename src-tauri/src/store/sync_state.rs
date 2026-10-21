//! What a connector last saw for each thing it syncs, so it can tell what changed upstream and
//! leave alone anything the user or an agent changed since.

use rusqlite::{Connection, OptionalExtension as _, params};

use crate::error::AppResult;

pub fn get(conn: &Connection, source: &str, external_id: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT snapshot FROM sync_state WHERE source = ?1 AND external_id = ?2",
            [source, external_id],
            |r| r.get(0),
        )
        .optional()?)
}

pub fn set(conn: &Connection, source: &str, external_id: &str, snapshot: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_state (source, external_id, snapshot) VALUES (?1, ?2, ?3)
         ON CONFLICT(source, external_id) DO UPDATE SET snapshot = excluded.snapshot",
        params![source, external_id, snapshot],
    )?;
    Ok(())
}

/// Forgets things that no longer exist upstream.
pub fn forget(conn: &Connection, source: &str, external_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM sync_state WHERE source = ?1 AND external_id = ?2",
        [source, external_id],
    )?;
    Ok(())
}
