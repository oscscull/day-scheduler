//! The agent session holding each item, so Quarterdeck can resume it.

use std::collections::HashMap;

use super::client;
use super::convert::text_key;
use super::db::{Db, gone_ok};
use crate::domain::session::SessionRef;
use crate::error::{AppError, AppResult};

/// The session holding each item that has one, by item id.
pub async fn all(db: &Db) -> AppResult<HashMap<String, SessionRef>> {
    let rows = db.api().watches(client::WatchesInput {}, ()).await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let cwd = r.cwd;
            let session = r.session.map(|id| SessionRef { id, cwd })?;
            Some((r.item_id.into_raw(), session))
        })
        .collect())
}

pub async fn session_for(db: &Db, item_id: &str) -> AppResult<Option<SessionRef>> {
    Ok(all(db).await?.remove(item_id))
}

/// Records the session working on an item, replacing any before it.
pub async fn claim(db: &Db, item_id: &str, session: &SessionRef) -> AppResult<()> {
    let input = client::ClaimWithSessionInput {
        item: text_key(item_id),
        session: session.id.clone(),
        cwd: session.cwd.clone(),
    };
    Ok(db.api().claim_with_session(input, ()).await?)
}

/// Forgets the session holding an item.
pub async fn release(db: &Db, item_id: &str) -> AppResult<()> {
    let input = client::ReleaseInput {
        item: text_key(item_id),
    };
    gone_ok(db.api().release(input, ()).await.map_err(AppError::from))
}
