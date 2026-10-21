use rusqlite::{Connection, Row, params};

use crate::domain::run::Run;
use crate::error::AppResult;

const COLUMNS: &str = "id, schedule_id, what, target, node, started_at, finished_at, summary";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Run> {
    Ok(Run {
        id: row.get(0)?,
        schedule_id: row.get(1)?,
        what: row.get(2)?,
        target: row.get(3)?,
        node: row.get(4)?,
        started_at: row.get(5)?,
        finished_at: row.get(6)?,
        summary: row.get(7)?,
    })
}

fn query(conn: &Connection, sql: &str, args: impl rusqlite::Params) -> AppResult<Vec<Run>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(args, from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Runs that have started but not finished.
pub fn live(conn: &Connection) -> AppResult<Vec<Run>> {
    let sql = format!("SELECT {COLUMNS} FROM runs WHERE finished_at IS NULL ORDER BY started_at");
    query(conn, &sql, [])
}

/// The latest finished runs of one schedule.
pub fn recent_for(conn: &Connection, schedule_id: &str, limit: u32) -> AppResult<Vec<Run>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM runs WHERE schedule_id = ?1 AND finished_at IS NOT NULL
         ORDER BY started_at DESC LIMIT ?2"
    );
    query(conn, &sql, params![schedule_id, limit])
}

/// Forgets the history of a deleted schedule.
pub fn delete_for(conn: &Connection, schedule_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM runs WHERE schedule_id = ?1",
        params![schedule_id],
    )?;
    Ok(())
}

pub fn insert(conn: &Connection, run: &Run) -> AppResult<()> {
    let sql = format!("INSERT INTO runs ({COLUMNS}) VALUES (NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7)");
    conn.execute(
        &sql,
        params![
            run.schedule_id,
            run.what,
            run.target,
            run.node,
            run.started_at,
            run.finished_at,
            run.summary
        ],
    )?;
    Ok(())
}
