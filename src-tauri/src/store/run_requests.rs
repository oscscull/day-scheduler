//! "Run this schedule now" requests from outside the app (e.g. the MCP server), picked up by the
//! scheduler on its next tick. Requests are deleted once taken.

use std::collections::BTreeSet;

use super::client;
use super::db::Db;
use crate::error::AppResult;

pub async fn add(db: &Db, schedule_id: &str) -> AppResult<()> {
    let input = client::RequestRunInput {
        schedule_id: schedule_id.to_owned(),
    };
    Ok(db.api().request_run(input, ()).await?)
}

/// The schedules asked for since the last take, each once.
pub async fn take_all(db: &Db) -> AppResult<Vec<String>> {
    db.tx(|api| async move {
        let rows = api.run_requests(client::RunRequestsInput {}, ()).await?;
        api.clear_run_requests(client::ClearRunRequestsInput {}, ())
            .await?;
        let ids: BTreeSet<String> = rows.into_iter().map(|r| r.schedule_id).collect();
        Ok(ids.into_iter().collect())
    })
    .await
}
