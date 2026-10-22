use chrono::{DateTime, Utc};

use super::client::{self, RunRow};
use super::convert::{int, key, same, same_opt};
use super::db::{Db, gone_ok};
use super::time;
use crate::domain::run::{Outcome, Run};
use crate::error::{AppError, AppResult};

fn from_row(row: RunRow) -> AppResult<Run> {
    Ok(Run {
        id: int(&row.id)?,
        schedule_id: row.schedule_id,
        what: row.what,
        target: row.target,
        node: row.node,
        started_at: time::get(&row.started_at)?,
        finished_at: time::get_opt(row.finished_at.as_deref())?,
        outcome: same_opt(row.outcome.as_ref())?,
        summary: row.summary,
    })
}

fn from_rows(rows: Vec<RunRow>) -> AppResult<Vec<Run>> {
    rows.into_iter().map(from_row).collect()
}

/// Runs that have started but not finished.
pub async fn live(db: &Db) -> AppResult<Vec<Run>> {
    from_rows(db.api().live_runs(client::LiveRunsInput {}, ()).await?)
}

/// Runs still marked as going when the app starts can't be: their process ended with the app.
pub async fn close_interrupted(db: &Db, at: DateTime<Utc>) -> AppResult<()> {
    let input = client::CloseInterruptedRunsInput { at: time::put(at) };
    Ok(db.api().close_interrupted_runs(input, ()).await?)
}

/// The latest finished runs of one schedule.
pub async fn recent_for(db: &Db, schedule_id: &str, limit: u32) -> AppResult<Vec<Run>> {
    let input = client::RecentRunsInput {
        schedule_id: schedule_id.to_owned(),
        limit: i64::from(limit),
    };
    from_rows(db.api().recent_runs(input, ()).await?)
}

/// Forgets the history of a deleted schedule.
pub async fn delete_for(db: &Db, schedule_id: &str) -> AppResult<()> {
    let input = client::ForgetRunsInput {
        schedule_id: schedule_id.to_owned(),
    };
    gone_ok(
        db.api()
            .forget_runs(input, ())
            .await
            .map_err(AppError::from),
    )
}

/// Records a run and returns its id.
pub async fn insert(db: &Db, run: &Run) -> AppResult<i64> {
    let input = client::AddRunInput {
        schedule_id: run.schedule_id.clone(),
        what: run.what.clone(),
        target: run.target.clone(),
        node: run.node.clone(),
        started_at: time::put(run.started_at),
        finished_at: time::put_opt(run.finished_at),
        outcome: same_opt(run.outcome.as_ref())?,
        summary: run.summary.clone(),
    };
    let row = db.api().add_run(input, ()).await?;
    int(&row.id)
}

/// Records that a run has started and returns its id.
pub async fn start(db: &Db, run: &Run) -> AppResult<i64> {
    insert(db, run).await
}

/// Marks a run finished now.
pub async fn finish(db: &Db, id: i64, outcome: Outcome, summary: &str) -> AppResult<()> {
    let input = client::FinishRunInput {
        id: key(id),
        at: time::put(Utc::now()),
        outcome: same(&outcome)?,
        summary: summary.to_owned(),
    };
    Ok(db.api().finish_run(input, ()).await?)
}
