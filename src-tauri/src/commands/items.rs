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

#[tauri::command]
pub fn add_ask(
    app: State<'_, App>,
    item_id: String,
    question: String,
    context: String,
) -> AppResult<()> {
    app.db
        .with(|conn| asks::insert(conn, &item_id, &question, &context))
}

#[tauri::command]
pub fn resolve_ask(app: State<'_, App>, id: i64) -> AppResult<()> {
    app.db.with(|conn| asks::resolve(conn, id))
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
