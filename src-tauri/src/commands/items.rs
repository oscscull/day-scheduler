use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::answer::Outcome;
use crate::app::App;
use crate::domain::ask::Ask;
use crate::domain::event::ItemEvent;
use crate::domain::item::{ItemInput, WorkItem};
use crate::error::AppResult;
use crate::store::{asks, events, items};
use crate::work;

/// Saves a new priority order; `ids` is every item, most important first.
#[tauri::command]
pub fn reorder_items(app: State<'_, App>, ids: Vec<String>) -> AppResult<()> {
    app.db.with(|conn| items::reorder(conn, &ids))
}

/// Creates or edits an item and returns its id.
#[tauri::command]
pub fn save_item(app: State<'_, App>, mut item: ItemInput) -> AppResult<String> {
    item.area = app.settings()?.known_area(&item.area);
    app.db.with(|conn| work::save(conn, item, Utc::now()))
}

#[tauri::command]
pub fn delete_item(app: State<'_, App>, id: String) -> AppResult<()> {
    app.db.with(|conn| items::delete(conn, &id))
}

/// What the user does about something that needs them.
#[derive(Debug, serde::Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Response {
    Answer {
        ask_id: i64,
        text: String,
    },
    /// Drops a question that no longer matters.
    Dismiss {
        ask_id: i64,
    },
    /// Tells Claude what to do about an item, with no question behind it.
    Tell {
        item_id: String,
        text: String,
    },
}

#[tauri::command]
pub fn respond(app: State<'_, App>, response: Response) -> AppResult<Outcome> {
    let settings = app.settings()?;
    app.db.with(|conn| match response {
        Response::Answer { ask_id, text } => crate::answer::give(conn, &settings, ask_id, &text),
        Response::Dismiss { ask_id } => asks::remove(conn, ask_id).map(|()| Outcome::Done),
        Response::Tell { item_id, text } => {
            crate::answer::tell(conn, &settings, &item_id, &text).map(|()| Outcome::Done)
        }
    })
}

/// One item with its open asks and history, for the detail view.
#[derive(Debug, Serialize)]
pub struct ItemDetail {
    item: WorkItem,
    asks: Vec<Ask>,
    history: Vec<ItemEvent>,
}

#[tauri::command]
pub fn item_detail(app: State<'_, App>, id: String) -> AppResult<ItemDetail> {
    app.db.with(|conn| {
        Ok(ItemDetail {
            item: items::get(conn, &id)?,
            asks: asks::for_item(conn, &id)?,
            history: events::for_item(conn, &id)?,
        })
    })
}
