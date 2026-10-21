//! Fills an empty database with the sample data in `defaults/seed.json`.
//! Times are relative to now, so the sample always looks current. Read by: the app on first run.
//! TTL: until real connectors fill the database.

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::Deserialize;

use super::{activity, asks, commitments, items, runs, todos, urgent};
use crate::domain::activity::Activity;
use crate::domain::commitment::Commitment;
use crate::domain::item::{Stage, Status, WorkItem};
use crate::domain::run::{Outcome, Run};
use crate::domain::todo::Todo;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;

const SEED: &str = include_str!("../../defaults/seed.json");

#[derive(Debug, Deserialize)]
struct Seed {
    items: Vec<SeedItem>,
    asks: Vec<SeedAsk>,
    commitments: Vec<SeedCommitment>,
    activity: Vec<SeedActivity>,
    runs: Vec<SeedRun>,
    urgent: Vec<SeedUrgent>,
    todos: Vec<SeedTodo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedItem {
    id: String,
    title: String,
    state_label: String,
    stage: Stage,
    status: Option<Status>,
    holder: String,
    area: String,
    opened_mins_ago: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedAsk {
    item_id: String,
    question: String,
    context: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedCommitment {
    title: String,
    area: String,
    starts_in_mins: i64,
    duration_mins: i64,
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedActivity {
    text: String,
    area: String,
    mins_ago: i64,
    unread: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedRun {
    schedule_id: Option<String>,
    what: String,
    target: String,
    node: String,
    started_mins_ago: i64,
    finished_mins_ago: Option<i64>,
    outcome: Option<Outcome>,
    summary: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedUrgent {
    title: String,
    detail: String,
    area: String,
    mins_ago: i64,
    schedule_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedTodo {
    title: String,
    area: String,
    due_in_days: Option<i64>,
}

fn ago(now: DateTime<Utc>, mins: i64) -> DateTime<Utc> {
    now - Duration::minutes(mins)
}

pub fn seed_if_empty(conn: &Connection, now: DateTime<Utc>) -> AppResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let seed: Seed = serde_json::from_str(SEED)?;
    for (rank, s) in (0_i64..).zip(seed.items) {
        items::insert(conn, &item(s, rank, now))?;
    }
    for a in seed.asks {
        asks::insert(conn, &a.item_id, &a.question, &a.context)?;
    }
    for c in seed.commitments {
        commitments::insert(conn, &commitment(c, now))?;
    }
    for a in seed.activity {
        activity::insert(conn, &note(a, now))?;
    }
    for u in seed.urgent {
        urgent::raise(conn, &urgent_item(u, now))?;
    }
    for t in seed.todos {
        todos::insert(conn, &todo(t, now))?;
    }
    for r in seed.runs {
        runs::insert(conn, &run(r, now))?;
    }
    Ok(())
}

fn item(s: SeedItem, rank: i64, now: DateTime<Utc>) -> WorkItem {
    WorkItem {
        id: s.id,
        title: s.title,
        state_label: s.state_label,
        stage: s.stage,
        status: s.status,
        holder: s.holder,
        area: s.area,
        opened_at: ago(now, s.opened_mins_ago),
        rank,
    }
}

fn commitment(s: SeedCommitment, now: DateTime<Utc>) -> Commitment {
    let starts_at = now + Duration::minutes(s.starts_in_mins);
    Commitment {
        id: 0,
        title: s.title,
        area: s.area,
        starts_at,
        ends_at: starts_at + Duration::minutes(s.duration_mins),
        location: s.location,
    }
}

fn note(s: SeedActivity, now: DateTime<Utc>) -> Activity {
    Activity {
        id: 0,
        text: s.text,
        area: s.area,
        at: ago(now, s.mins_ago),
        unread: s.unread,
    }
}

fn urgent_item(s: SeedUrgent, now: DateTime<Utc>) -> Urgent {
    Urgent {
        id: 0,
        title: s.title,
        detail: s.detail,
        area: s.area,
        raised_at: ago(now, s.mins_ago),
        schedule_id: s.schedule_id,
    }
}

fn todo(s: SeedTodo, now: DateTime<Utc>) -> Todo {
    let due = s
        .due_in_days
        .map(|d| (now + Duration::days(d)).date_naive());
    Todo {
        id: 0,
        title: s.title,
        area: s.area,
        due,
        done_at: None,
    }
}

fn run(s: SeedRun, now: DateTime<Utc>) -> Run {
    Run {
        id: 0,
        schedule_id: s.schedule_id,
        what: s.what,
        target: s.target,
        node: s.node,
        started_at: ago(now, s.started_mins_ago),
        finished_at: s.finished_mins_ago.map(|m| ago(now, m)),
        outcome: s.outcome,
        summary: s.summary,
    }
}
