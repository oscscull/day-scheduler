use super::client::{self, CommitmentRow};
use super::convert::{int, key};
use super::db::{Db, gone_ok};
use super::time;
use crate::domain::commitment::Commitment;
use crate::error::{AppError, AppResult};

fn from_row(row: CommitmentRow) -> AppResult<Commitment> {
    Ok(Commitment {
        id: int(&row.id)?,
        title: row.title,
        area: row.area,
        starts_at: time::get(&row.starts_at)?,
        ends_at: time::get(&row.ends_at)?,
        location: row.location,
        join_url: row.join_url,
        source: row.source,
        external_id: row.external_id,
        hidden: row.hidden,
    })
}

fn from_rows(rows: Vec<CommitmentRow>) -> AppResult<Vec<Commitment>> {
    rows.into_iter().map(from_row).collect()
}

/// Commitments that are not hidden, soonest first.
pub async fn list(db: &Db) -> AppResult<Vec<Commitment>> {
    from_rows(
        db.api()
            .commitments(client::CommitmentsInput {}, ())
            .await?,
    )
}

/// Everything from one source, hidden ones included, for syncing.
pub async fn from_source(db: &Db, source: &str) -> AppResult<Vec<Commitment>> {
    let input = client::CommitmentsFromInput {
        source: source.to_owned(),
    };
    from_rows(db.api().commitments_from(input, ()).await?)
}

pub async fn by_external(
    db: &Db,
    source: &str,
    external_id: &str,
) -> AppResult<Option<Commitment>> {
    let input = client::CommitmentByExternalInput {
        source: source.to_owned(),
        external_id: external_id.to_owned(),
    };
    let row = db.api().commitment_by_external(input, ()).await?;
    row.map(from_row).transpose()
}

pub async fn get(db: &Db, id: i64) -> AppResult<Commitment> {
    let input = client::CommitmentInput { id: key(id) };
    let row = db.api().commitment(input, ()).await?;
    from_row(row.ok_or_else(|| AppError::NotFound(format!("meeting {id}")))?)
}

/// Stores a new commitment; its `id` is ignored and assigned by the database.
pub async fn insert(db: &Db, c: &Commitment) -> AppResult<()> {
    let input = client::AddCommitmentInput {
        title: c.title.clone(),
        area: c.area.clone(),
        starts_at: time::put(c.starts_at),
        ends_at: time::put(c.ends_at),
        location: c.location.clone(),
        join_url: c.join_url.clone(),
        source: c.source.clone(),
        external_id: c.external_id.clone(),
        hidden: c.hidden,
    };
    Ok(db.api().add_commitment(input, ()).await?)
}

pub async fn update(db: &Db, c: &Commitment) -> AppResult<()> {
    let input = client::UpdateCommitmentInput {
        id: key(c.id),
        title: c.title.clone(),
        area: c.area.clone(),
        starts_at: time::put(c.starts_at),
        ends_at: time::put(c.ends_at),
        location: c.location.clone(),
        join_url: c.join_url.clone(),
        hidden: c.hidden,
    };
    Ok(db.api().update_commitment(input, ()).await?)
}

/// Deletes a meeting and its alert state.
pub async fn delete(db: &Db, id: i64) -> AppResult<()> {
    let input = client::DeleteCommitmentInput { id: key(id) };
    gone_ok(
        db.api()
            .delete_commitment(input, ())
            .await
            .map_err(AppError::from),
    )
}
