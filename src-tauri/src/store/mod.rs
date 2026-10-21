//! `SQLite` storage for live state: items, asks, meetings, activity and runs.
//! Behaviour (schedules, prompts, policies) lives in files instead; see `config`.

mod codec;
pub mod schema;

pub mod activity;
pub mod alert_state;
pub mod asks;
pub mod commitments;
pub mod events;
pub mod expire;
pub mod items;
pub mod meta;
pub mod run_requests;
pub mod runs;
pub mod sync_state;
pub mod todos;
pub mod urgent;
pub mod watch;

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

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
        // The app and the MCP server share this file, so wait for each other's writes.
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
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
