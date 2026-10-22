//! Watchdog state per item Claude holds: its session (to resume) and check-ins so far.
//! Rows go with their item.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::client;
use super::convert::text_key;
use super::db::{Db, gone_ok};
use super::time;
use crate::domain::session::SessionRef;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Watch {
    /// The Claude Code session working on it, if the agent said.
    pub session: Option<SessionRef>,
    pub nudges: u32,
    pub last_nudge: Option<DateTime<Utc>>,
}

pub async fn all(db: &Db) -> AppResult<HashMap<String, Watch>> {
    let rows = db.api().watches(client::WatchesInput {}, ()).await?;
    rows.into_iter()
        .map(|r| {
            let cwd = r.cwd;
            let watch = Watch {
                session: r.session.map(|id| SessionRef { id, cwd }),
                nudges: u32::try_from(r.nudges).unwrap_or_default(),
                last_nudge: time::get_opt(r.last_nudge.as_deref())?,
            };
            Ok((r.item_id.into_raw(), watch))
        })
        .collect()
}

/// An agent took the item on; a fresh start for the watchdog, keeping any session already known.
pub async fn claim(db: &Db, item_id: &str, session: Option<&SessionRef>) -> AppResult<()> {
    let api = db.api();
    let item = text_key(item_id);
    if let Some(s) = session {
        let input = client::ClaimWithSessionInput {
            item,
            session: s.id.clone(),
            cwd: s.cwd.clone(),
        };
        return Ok(api.claim_with_session(input, ()).await?);
    }
    Ok(api.claim(client::ClaimInput { item }, ()).await?)
}

pub async fn record(db: &Db, item_id: &str, w: &Watch) -> AppResult<()> {
    let input = client::RecordNudgesInput {
        item: text_key(item_id),
        nudges: i64::from(w.nudges),
        last_nudge: time::put_opt(w.last_nudge),
    };
    Ok(db.api().record_nudges(input, ()).await?)
}

/// Stops watching an item once nobody is carrying it.
pub async fn release(db: &Db, item_id: &str) -> AppResult<()> {
    let input = client::ReleaseInput {
        item: text_key(item_id),
    };
    gone_ok(db.api().release(input, ()).await.map_err(AppError::from))
}
