//! Watchdog state per item Claude holds: its session (to resume) and check-ins so far.
//! Rows go with their item.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::error::AppResult;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Watch {
    /// The Claude Code session working on it, if the agent said.
    pub session: Option<String>,
    pub nudges: u32,
    pub last_nudge: Option<DateTime<Utc>>,
}

pub fn all(conn: &Connection) -> AppResult<HashMap<String, Watch>> {
    let mut stmt = conn.prepare("SELECT item_id, session, nudges, last_nudge FROM watch")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get(0)?,
            Watch {
                session: r.get(1)?,
                nudges: r.get(2)?,
                last_nudge: r.get(3)?,
            },
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// An agent took the item on; a fresh start for the watchdog.
pub fn claim(conn: &Connection, item_id: &str, session: Option<&str>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO watch (item_id, session, nudges, last_nudge) VALUES (?1, ?2, 0, NULL)
         ON CONFLICT(item_id) DO UPDATE SET session = COALESCE(excluded.session, watch.session),
         nudges = 0, last_nudge = NULL",
        params![item_id, session],
    )?;
    Ok(())
}

pub fn record(conn: &Connection, item_id: &str, w: &Watch) -> AppResult<()> {
    conn.execute(
        "INSERT INTO watch (item_id, session, nudges, last_nudge) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(item_id) DO UPDATE SET nudges = excluded.nudges, last_nudge = excluded.last_nudge",
        params![item_id, w.session, w.nudges, w.last_nudge],
    )?;
    Ok(())
}

/// Stops watching an item once nobody is carrying it.
pub fn release(conn: &Connection, item_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM watch WHERE item_id = ?1", [item_id])?;
    Ok(())
}
