//! When each connector last synced and whether it worked, for the Agent tab.

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;

use crate::error::AppResult;
use crate::store::meta;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub name: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

fn key(name: &str, part: &str) -> String {
    format!("connector.{name}.{part}")
}

pub fn record(conn: &Connection, name: &str, error: Option<&str>) -> AppResult<()> {
    meta::set_time(conn, &key(name, "last_sync"), Utc::now())?;
    meta::set_text(conn, &key(name, "error"), error.unwrap_or_default())
}

pub fn read(conn: &Connection, name: &str) -> AppResult<Status> {
    Ok(Status {
        name: name.to_owned(),
        last_sync: meta::time(conn, &key(name, "last_sync"))?,
        error: meta::text(conn, &key(name, "error"))?.filter(|e| !e.is_empty()),
    })
}

/// Clears the last sync so the connector runs on the next tick, e.g. after its settings change.
pub fn sync_soon(conn: &Connection, name: &str) -> AppResult<()> {
    meta::forget(conn, &key(name, "last_sync"))?;
    meta::forget(conn, &key(name, "error"))
}
