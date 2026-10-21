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
    area TEXT NOT NULL,
    link TEXT,
    source TEXT NOT NULL DEFAULT 'manual',
    external_id TEXT,
    hidden INTEGER NOT NULL DEFAULT 0,
    opened_at TEXT NOT NULL,
    state_since TEXT NOT NULL,
    rank INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS asks (
    id INTEGER PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items(id),
    question TEXT NOT NULL,
    context TEXT NOT NULL,
    options TEXT NOT NULL DEFAULT '[]',
    session TEXT,
    cwd TEXT,
    answer TEXT
);
CREATE TABLE IF NOT EXISTS commitments (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    area TEXT NOT NULL,
    starts_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    location TEXT,
    join_url TEXT,
    source TEXT NOT NULL,
    external_id TEXT,
    hidden INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS activity (
    id INTEGER PRIMARY KEY,
    text TEXT NOT NULL,
    area TEXT NOT NULL,
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
    outcome TEXT,
    summary TEXT
);
CREATE TABLE IF NOT EXISTS urgent (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    detail TEXT NOT NULL,
    area TEXT NOT NULL,
    raised_at TEXT NOT NULL,
    schedule_id TEXT
);
CREATE TABLE IF NOT EXISTS item_events (
    id INTEGER PRIMARY KEY,
    item_id TEXT NOT NULL,
    at TEXT NOT NULL,
    change TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS run_requests (
    schedule_id TEXT NOT NULL,
    requested_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sync_state (
    source TEXT NOT NULL,
    external_id TEXT NOT NULL,
    snapshot TEXT NOT NULL,
    PRIMARY KEY (source, external_id)
);
CREATE TABLE IF NOT EXISTS alert_state (
    commitment_id INTEGER PRIMARY KEY,
    done INTEGER NOT NULL DEFAULT 0,
    snoozed_until TEXT
);
CREATE TABLE IF NOT EXISTS watch (
    item_id TEXT PRIMARY KEY,
    session TEXT,
    cwd TEXT,
    nudges INTEGER NOT NULL DEFAULT 0,
    last_nudge TEXT
);
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    area TEXT NOT NULL,
    due TEXT,
    done_at TEXT
);
";

/// Changes to databases made before the current schema, in order. Append only: each runs once,
/// tracked by `SQLite`'s `user_version`. A fresh database starts from `SCHEMA` and skips them all.
const MIGRATIONS: [&str; 6] = [
    "ALTER TABLE commitments ADD COLUMN join_url TEXT",
    "ALTER TABLE asks ADD COLUMN options TEXT NOT NULL DEFAULT '[]'",
    "ALTER TABLE asks ADD COLUMN session TEXT",
    "ALTER TABLE asks ADD COLUMN cwd TEXT",
    "ALTER TABLE asks ADD COLUMN answer TEXT",
    "ALTER TABLE watch ADD COLUMN cwd TEXT",
];

fn version(conn: &Connection) -> AppResult<usize> {
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    Ok(usize::try_from(v).unwrap_or(0))
}

fn set_version(conn: &Connection, v: usize) -> AppResult<()> {
    conn.pragma_update(None, "user_version", i64::try_from(v).unwrap_or(i64::MAX))?;
    Ok(())
}

/// Runs one migration. A table created after a column joined `SCHEMA` already has that column, so
/// adding it again counts as done.
fn apply(conn: &Connection, sql: &str) -> AppResult<()> {
    match conn.execute_batch(sql) {
        Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
            if msg.starts_with("duplicate column name") =>
        {
            Ok(())
        }
        other => Ok(other?),
    }
}

pub fn migrate(conn: &Connection) -> AppResult<()> {
    let fresh: bool = conn.query_row(
        "SELECT COUNT(*) = 0 FROM sqlite_master WHERE type = 'table' AND name = 'items'",
        [],
        |r| r.get(0),
    )?;
    conn.execute_batch(SCHEMA)?;
    if fresh {
        return set_version(conn, MIGRATIONS.len());
    }
    for (done, sql) in MIGRATIONS.iter().enumerate().skip(version(conn)?) {
        apply(conn, sql)?;
        set_version(conn, done + 1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_databases_catch_up_and_fresh_ones_start_current() {
        let fresh = Connection::open_in_memory().unwrap();
        migrate(&fresh).unwrap();
        migrate(&fresh).unwrap();
        assert_eq!(version(&fresh).unwrap(), MIGRATIONS.len());
        let old = Connection::open_in_memory().unwrap();
        old.execute_batch(&SCHEMA.replace(
            "    location TEXT,\n    join_url TEXT,",
            "    location TEXT,",
        ))
        .unwrap();
        migrate(&old).unwrap();
        old.execute("UPDATE commitments SET join_url = 'x'", [])
            .unwrap();
        old.execute("UPDATE asks SET options = '[]', answer = NULL", [])
            .unwrap();
        assert_eq!(version(&old).unwrap(), MIGRATIONS.len());
    }
}
