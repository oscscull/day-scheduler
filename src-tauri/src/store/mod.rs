//! `SQLite` storage for live state: items, asks, meetings, activity and runs.
//! Behaviour (schedules, prompts, policies) lives in files instead; see `config`.

mod codec;
pub mod schema;

pub mod activity;
pub mod asks;
pub mod commitments;
pub mod events;
pub mod expire;
pub mod items;
pub mod meta;
pub mod runs;
pub mod todos;
pub mod urgent;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// The app's database handle, shared through Tauri state.
#[derive(Debug)]
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Opens (or creates) the database in `dir` and brings its tables up to date.
    pub fn open(dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(dir)?;
        let conn = Connection::open(dir.join("bridge.sqlite"))?;
        schema::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Runs `f` with exclusive access to the connection.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.conn.lock().map_err(|_| AppError::Lock)?;
        f(&conn)
    }
}
