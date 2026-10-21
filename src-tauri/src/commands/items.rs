use tauri::State;

use crate::app::App;
use crate::error::AppResult;
use crate::store::items;

/// Saves a new priority order; `ids` is every item, most important first.
#[tauri::command]
pub fn reorder_items(app: State<'_, App>, ids: Vec<String>) -> AppResult<()> {
    app.db.with(|conn| items::reorder(conn, &ids))
}
