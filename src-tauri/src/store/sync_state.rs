//! What a connector last saw for each thing it syncs, so it can tell what changed upstream and
//! leave alone anything the user or an agent changed since.

use super::client;
use super::db::{Db, gone_ok};
use crate::error::{AppError, AppResult};

pub async fn get(db: &Db, source: &str, external_id: &str) -> AppResult<Option<String>> {
    let input = client::SyncSnapshotInput {
        source: source.to_owned(),
        external_id: external_id.to_owned(),
    };
    let row = db.api().sync_snapshot(input, ()).await?;
    Ok(row.map(|r| r.snapshot))
}

pub async fn set(db: &Db, source: &str, external_id: &str, snapshot: &str) -> AppResult<()> {
    let input = client::SetSyncSnapshotInput {
        source: source.to_owned(),
        external_id: external_id.to_owned(),
        snapshot: snapshot.to_owned(),
    };
    Ok(db.api().set_sync_snapshot(input, ()).await?)
}

/// Forgets things that no longer exist upstream.
pub async fn forget(db: &Db, source: &str, external_id: &str) -> AppResult<()> {
    let input = client::ForgetSyncSnapshotInput {
        source: source.to_owned(),
        external_id: external_id.to_owned(),
    };
    gone_ok(
        db.api()
            .forget_sync_snapshot(input, ())
            .await
            .map_err(AppError::from),
    )
}
