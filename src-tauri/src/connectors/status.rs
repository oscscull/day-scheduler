//! When each connector last synced and whether it worked, for the Agent tab.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::error::AppResult;
use crate::store::{Db, meta};

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

pub async fn record(db: &Db, name: &str, error: Option<&str>) -> AppResult<()> {
    meta::set_time(db, &key(name, "last_sync"), Utc::now()).await?;
    meta::set_text(db, &key(name, "error"), error.unwrap_or_default()).await
}

pub async fn read(db: &Db, name: &str) -> AppResult<Status> {
    Ok(Status {
        name: name.to_owned(),
        last_sync: meta::time(db, &key(name, "last_sync")).await?,
        error: meta::text(db, &key(name, "error"))
            .await?
            .filter(|e| !e.is_empty()),
    })
}

/// Clears the last sync so the connector runs on the next tick, e.g. after its settings change.
pub async fn sync_soon(db: &Db, name: &str) -> AppResult<()> {
    meta::forget(db, &key(name, "last_sync")).await?;
    meta::forget(db, &key(name, "error")).await
}
