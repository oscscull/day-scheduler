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

/// What the edit form sends: the whole to-do, so clearing the date clears it.
#[derive(Debug, serde::Deserialize)]
pub struct TodoEdit {
    id: i64,
    title: String,
    area: String,
    due: Option<NaiveDate>,
}

#[tauri::command]
pub fn update_todo(app: State<'_, App>, todo: TodoEdit) -> AppResult<()> {
    let area = app.settings()?.known_area(&todo.area);
    app.db.with(|conn| {
        let old = todos::get(conn, todo.id)?;
        let new = Todo {
            title: todo.title,
            area,
            due: todo.due,
            ..old
        };
        todos::update(conn, &new)
    })
}

#[tauri::command]
pub fn delete_todo(app: State<'_, App>, id: i64) -> AppResult<()> {
    app.db.with(|conn| todos::delete(conn, id))
}
