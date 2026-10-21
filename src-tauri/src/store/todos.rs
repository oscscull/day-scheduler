use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension as _, Row, params};

use crate::domain::todo::Todo;
use crate::error::{AppError, AppResult};

fn from_row(row: &Row<'_>) -> rusqlite::Result<Todo> {
    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        area: row.get(2)?,
        due: row.get(3)?,
        done_at: row.get(4)?,
    })
}

/// Open to-dos (soonest due first, undated last), then recently done ones.
pub fn list(conn: &Connection) -> AppResult<Vec<Todo>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, area, due, done_at FROM todos
         ORDER BY done_at IS NOT NULL, due IS NULL, due, id",
    )?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Stores a new to-do; its `id` is ignored and assigned by the database.
pub fn insert(conn: &Connection, todo: &Todo) -> AppResult<()> {
    conn.execute(
        "INSERT INTO todos (title, area, due, done_at) VALUES (?1, ?2, ?3, ?4)",
        params![todo.title, todo.area, todo.due, todo.done_at],
    )?;
    Ok(())
}

pub fn set_done(conn: &Connection, id: i64, done_at: Option<DateTime<Utc>>) -> AppResult<()> {
    conn.execute(
        "UPDATE todos SET done_at = ?1 WHERE id = ?2",
        params![done_at, id],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Todo> {
    conn.query_row(
        "SELECT id, title, area, due, done_at FROM todos WHERE id = ?1",
        [id],
        from_row,
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound(format!("to-do {id}")))
}

pub fn update(conn: &Connection, todo: &Todo) -> AppResult<()> {
    conn.execute(
        "UPDATE todos SET title = ?1, area = ?2, due = ?3, done_at = ?4 WHERE id = ?5",
        params![todo.title, todo.area, todo.due, todo.done_at, todo.id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
    Ok(())
}
