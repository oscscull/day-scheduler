//! What an item's history says about its turn.

use chrono::{DateTime, Utc};

use crate::domain::event::Change;
use crate::error::AppResult;
use crate::store::{Db, events};

/// Why the item last changed hands, if it ever did.
pub async fn last_why(db: &Db, item_id: &str) -> AppResult<Option<String>> {
    let history = events::for_item(db, item_id).await?;
    Ok(history.into_iter().find_map(|e| match e.change {
        Change::Turn { why, .. } => Some(why),
        Change::Created
        | Change::Stage { .. }
        | Change::Status { .. }
        | Change::Label { .. }
        | Change::Note { .. } => None,
    }))
}

/// Whether the turn was passed by anyone since `since`.
pub async fn passed_since(db: &Db, item_id: &str, since: DateTime<Utc>) -> AppResult<bool> {
    let history = events::for_item(db, item_id).await?;
    Ok(history
        .iter()
        .any(|e| e.at > since && matches!(e.change, Change::Turn { .. })))
}
