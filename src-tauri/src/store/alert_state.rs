//! Whether each meeting's alert was dealt with or snoozed. Rows go when their meeting does.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct State {
    pub done: bool,
    pub snoozed_until: Option<DateTime<Utc>>,
}

pub fn all(conn: &Connection) -> AppResult<HashMap<i64, State>> {
    let mut stmt = conn.prepare("SELECT commitment_id, done, snoozed_until FROM alert_state")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get(0)?,
            State {
                done: r.get(1)?,
                snoozed_until: r.get(2)?,
            },
        ))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Joined or skipped: don't alert for this meeting again.
pub fn done(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO alert_state (commitment_id, done) VALUES (?1, 1)
         ON CONFLICT(commitment_id) DO UPDATE SET done = 1",
        [id],
    )?;
    Ok(())
}

pub fn snooze(conn: &Connection, id: i64, until: DateTime<Utc>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO alert_state (commitment_id, snoozed_until) VALUES (?1, ?2)
         ON CONFLICT(commitment_id) DO UPDATE SET snoozed_until = excluded.snoozed_until",
        params![id, until],
    )?;
    Ok(())
}
