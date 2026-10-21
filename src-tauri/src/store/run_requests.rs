//! "Run this schedule now" requests from outside the app (e.g. the MCP server), picked up by the
//! scheduler on its next tick. Each request is deleted once taken.

use chrono::Utc;
use rusqlite::{Connection, params};

use crate::error::AppResult;

pub fn add(conn: &Connection, schedule_id: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO run_requests (schedule_id, requested_at) VALUES (?1, ?2)",
        params![schedule_id, Utc::now()],
    )?;
    Ok(())
}

pub fn take_all(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT schedule_id FROM run_requests")?;
    let ids = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    conn.execute("DELETE FROM run_requests", [])?;
    Ok(ids)
}
