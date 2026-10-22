use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::app::App;
use crate::domain::ask::Ask;
use crate::domain::event::ItemEvent;
use crate::domain::item::{ItemInput, WorkItem};
use crate::error::AppResult;
use crate::store::{asks, events, items};
use crate::work;

/// Saves a new priority order; `ids` is every item, most important first.
#[tauri::command]
pub async fn reorder_items(app: State<'_, App>, ids: Vec<String>) -> AppResult<()> {
    items::reorder(&app.db, &ids).await
}

/// Creates or edits an item and returns its id.
#[tauri::command]
pub async fn save_item(app: State<'_, App>, mut item: ItemInput) -> AppResult<String> {
    item.area = app.settings()?.known_area(&item.area);
    work::save(&app.db, item, Utc::now()).await
}

#[tauri::command]
pub async fn delete_item(app: State<'_, App>, id: String) -> AppResult<()> {
    items::delete(&app.db, &id).await
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
pub async fn respond(app: State<'_, App>, response: Response) -> AppResult<()> {
    let settings = app.settings()?;
    let db = &app.db;
    match response {
        Response::Answer { ask_id, text } => {
            crate::answer::give(db, &settings, ask_id, &text).await
        }
        Response::Dismiss { ask_id } => asks::remove(db, ask_id).await,
        Response::Tell { item_id, text } => {
            crate::answer::tell(db, &settings, &item_id, &text).await
        }
    }
}

/// One item with its open asks and history, for the detail view.
#[derive(Debug, Serialize)]
pub struct ItemDetail {
    item: WorkItem,
    asks: Vec<Ask>,
    history: Vec<ItemEvent>,
}

#[tauri::command]
pub async fn item_detail(app: State<'_, App>, id: String) -> AppResult<ItemDetail> {
    Ok(ItemDetail {
        item: items::get(&app.db, &id).await?,
        asks: asks::for_item(&app.db, &id).await?,
        history: events::for_item(&app.db, &id).await?,
    })
}
