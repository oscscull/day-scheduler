//! Fills an empty database with the sample data in `defaults/seed.json`.
//! Times are relative to now, so the sample always looks current.

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::Deserialize;

use super::{activity, asks, commitments, items, runs, urgent};
use crate::domain::commitment::Commitment;
use crate::domain::item::{Stage, Status, WorkItem};
use crate::domain::run::{Outcome, Run};
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedUrgent {
    title: String,
    detail: String,
    mins_ago: i64,
    schedule_id: Option<String>,
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
    node: String,
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
    starts_in_mins: i64,
    duration_mins: i64,
    location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedActivity {
    text: String,
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

fn ago(now: DateTime<Utc>, mins: i64) -> DateTime<Utc> {
    now - Duration::minutes(mins)
}

pub fn seed_if_empty(conn: &Connection, now: DateTime<Utc>) -> AppResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let seed: Seed = serde_json::from_str(SEED)?;
    seed_items(conn, seed.items, now)?;
    for a in seed.asks {
        asks::insert(conn, &a.item_id, &a.question, &a.context)?;
    }
    seed_commitments(conn, seed.commitments, now)?;
    for a in seed.activity {
        activity::insert(conn, &a.text, ago(now, a.mins_ago), a.unread)?;
    }
    for u in seed.urgent {
        let item = Urgent {
            id: 0,
            title: u.title,
            detail: u.detail,
            raised_at: ago(now, u.mins_ago),
            schedule_id: u.schedule_id,
        };
        urgent::raise(conn, &item)?;
    }
    seed_runs(conn, seed.runs, now)
}

fn seed_items(conn: &Connection, list: Vec<SeedItem>, now: DateTime<Utc>) -> AppResult<()> {
    for (rank, s) in (0_i64..).zip(list) {
        let item = WorkItem {
            id: s.id,
            title: s.title,
            state_label: s.state_label,
            stage: s.stage,
            status: s.status,
            holder: s.holder,
            node: s.node,
            opened_at: ago(now, s.opened_mins_ago),
            rank,
        };
        items::insert(conn, &item)?;
    }
    Ok(())
}

fn seed_commitments(
    conn: &Connection,
    list: Vec<SeedCommitment>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    for s in list {
        let starts_at = now + Duration::minutes(s.starts_in_mins);
        let c = Commitment {
            id: 0,
            title: s.title,
            starts_at,
            ends_at: starts_at + Duration::minutes(s.duration_mins),
            location: s.location,
        };
        commitments::insert(conn, &c)?;
    }
    Ok(())
}

fn seed_runs(conn: &Connection, list: Vec<SeedRun>, now: DateTime<Utc>) -> AppResult<()> {
    for s in list {
        let run = Run {
            id: 0,
            schedule_id: s.schedule_id,
            what: s.what,
            target: s.target,
            node: s.node,
            started_at: ago(now, s.started_mins_ago),
            finished_at: s.finished_mins_ago.map(|m| ago(now, m)),
            outcome: s.outcome,
            summary: s.summary,
        };
        runs::insert(conn, &run)?;
    }
    Ok(())
}
