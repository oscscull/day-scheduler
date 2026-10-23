use chrono::Utc;
use serde::Serialize;
use tauri::State;

use crate::answer::Outcome;
use crate::app::App;
use crate::domain::ask::Ask;
use crate::domain::event::ItemEvent;
use crate::domain::item::{ItemInput, WorkItem};
use crate::error::AppResult;
use crate::park::{self, Parking};
use crate::store::{asks, events, items};
use crate::turns::Clock;
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

/// Parks started work until `until` (RFC 3339), with the user's reason if they gave one.
#[tauri::command]
pub async fn park_item(
    app: State<'_, App>,
    id: String,
    until: String,
    why: Option<String>,
) -> AppResult<()> {
    let until = chrono::DateTime::parse_from_rfc3339(&until)
        .map_err(|e| crate::error::AppError::BadInput(format!("'{until}' is not a time: {e}")))?
        .with_timezone(&Utc);
    let parking = Parking {
        until: Some(until),
        wake: Vec::new(),
        why,
    };
    let limits = app.settings()?.turns;
    park::park(&app.db, &id, parking, Clock::now(&limits)).await
}

/// Brings parked work back to the user now.
#[tauri::command]
pub async fn unpark_item(app: State<'_, App>, id: String) -> AppResult<()> {
    let limits = app.settings()?.turns;
    park::unpark(&app.db, &id, Clock::now(&limits)).await
}

/// Parked items, for the board's parked list.
#[tauri::command]
pub async fn parked_items(app: State<'_, App>) -> AppResult<Vec<WorkItem>> {
    park::list(&app.db).await
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
pub async fn respond(app: State<'_, App>, response: Response) -> AppResult<Outcome> {
    let settings = app.settings()?;
    let db = &app.db;
    match response {
        Response::Answer { ask_id, text } => {
            crate::answer::give(db, &settings, ask_id, &text).await
        }
        Response::Dismiss { ask_id } => asks::remove(db, ask_id).await.map(|()| Outcome::Done),
        Response::Tell { item_id, text } => crate::answer::tell(db, &settings, &item_id, &text)
            .await
            .map(|()| Outcome::Done),
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
