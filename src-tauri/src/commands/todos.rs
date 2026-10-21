use chrono::{NaiveDate, Utc};
use tauri::State;

use crate::app::App;
use crate::domain::todo::Todo;
use crate::error::AppResult;
use crate::store::todos;

#[tauri::command]
pub fn add_todo(
    app: State<'_, App>,
    title: String,
    area: String,
    due: Option<NaiveDate>,
) -> AppResult<()> {
    let area = app.settings()?.known_area(&area);
    let todo = Todo {
        id: 0,
        title,
        area,
        due,
        done_at: None,
    };
    app.db.with(|conn| todos::insert(conn, &todo))
}

/// Ticking a to-do off keeps it briefly, so a mis-click can be undone; expiry removes it later.
#[tauri::command]
pub fn set_todo_done(app: State<'_, App>, id: i64, done: bool) -> AppResult<()> {
    let at = done.then(Utc::now);
    app.db.with(|conn| todos::set_done(conn, id, at))
}
