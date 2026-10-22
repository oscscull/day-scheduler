use chrono::{DateTime, Utc};

use super::client::{self, EventRow};
use super::convert::{int, text_key};
use super::db::Db;
use super::time;
use crate::domain::event::{Change, ItemEvent};
use crate::error::AppResult;

fn from_row(row: EventRow) -> AppResult<ItemEvent> {
    Ok(ItemEvent {
        id: int(&row.id)?,
        item_id: row.item_id.into_raw(),
        at: time::get(&row.at)?,
        change: serde_json::from_value(row.change)?,
    })
}

/// An item's history, newest first.
pub async fn for_item(db: &Db, item_id: &str) -> AppResult<Vec<ItemEvent>> {
    let input = client::ItemEventsInput {
        item: text_key(item_id),
    };
    let rows = db.api().item_events(input, ()).await?;
    rows.into_iter().map(from_row).collect()
}

pub async fn record(db: &Db, item_id: &str, at: DateTime<Utc>, change: &Change) -> AppResult<()> {
    let input = client::RecordEventInput {
        item: text_key(item_id),
        at: time::put(at),
        change: serde_json::to_value(change)?,
    };
    Ok(db.api().record_event(input, ()).await?)
}
