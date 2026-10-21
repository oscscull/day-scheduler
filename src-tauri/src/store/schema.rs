use rusqlite::Connection;

use crate::error::AppResult;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    state_label TEXT NOT NULL,
    stage TEXT NOT NULL,
    status TEXT,
    holder TEXT NOT NULL,
    node TEXT NOT NULL,
    opened_at TEXT NOT NULL,
    rank INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS asks (
    id INTEGER PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items(id),
    question TEXT NOT NULL,
    context TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS commitments (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    location TEXT
);
CREATE TABLE IF NOT EXISTS activity (
    id INTEGER PRIMARY KEY,
    text TEXT NOT NULL,
    at TEXT NOT NULL,
    unread INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS runs (
    id INTEGER PRIMARY KEY,
    schedule_id TEXT,
    what TEXT NOT NULL,
    target TEXT NOT NULL,
    node TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    summary TEXT
);
";

pub fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}
