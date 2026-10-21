//! Watchdog state per item Claude holds: its session (to resume) and check-ins so far.
//! Rows go with their item.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::domain::session::SessionRef;
use crate::error::AppResult;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Watch {
    /// The Claude Code session working on it, if the agent said.
    pub session: Option<SessionRef>,
    pub nudges: u32,
    pub last_nudge: Option<DateTime<Utc>>,
}

pub fn all(conn: &Connection) -> AppResult<HashMap<String, Watch>> {
    let mut stmt = conn.prepare("SELECT item_id, session, cwd, nudges, last_nudge FROM watch")?;
    let rows = stmt.query_map([], |r| {
        let id: Option<String> = r.get(1)?;
        let cwd: Option<String> = r.get(2)?;
        Ok((
            r.get(0)?,
            Watch {
                session: id.map(|id| SessionRef { id, cwd }),
                nudges: r.get(3)?,
                last_nudge: r.get(4)?,
            },
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// An agent took the item on; a fresh start for the watchdog.
pub fn claim(conn: &Connection, item_id: &str, session: Option<&SessionRef>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO watch (item_id, session, cwd, nudges, last_nudge) VALUES (?1, ?2, ?3, 0, NULL)
         ON CONFLICT(item_id) DO UPDATE SET session = COALESCE(excluded.session, watch.session),
         cwd = CASE WHEN excluded.session IS NULL THEN watch.cwd ELSE excluded.cwd END,
         nudges = 0, last_nudge = NULL",
        params![
            item_id,
            session.map(|s| s.id.as_str()),
            session.and_then(|s| s.cwd.as_deref())
        ],
    )?;
    Ok(())
}

pub fn record(conn: &Connection, item_id: &str, w: &Watch) -> AppResult<()> {
    conn.execute(
        "INSERT INTO watch (item_id, nudges, last_nudge) VALUES (?1, ?2, ?3)
         ON CONFLICT(item_id) DO UPDATE SET nudges = excluded.nudges, last_nudge = excluded.last_nudge",
        params![item_id, w.nudges, w.last_nudge],
    )?;
    Ok(())
}

/// Stops watching an item once nobody is carrying it.
pub fn release(conn: &Connection, item_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM watch WHERE item_id = ?1", [item_id])?;
    Ok(())
}
