-- The pre-Based schema, for testing the import. Goes with the import.
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
