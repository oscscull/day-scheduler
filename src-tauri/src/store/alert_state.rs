//! Whether each meeting's alert was dealt with or snoozed. Rows go when their meeting does.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::client;
use super::convert::{int, key};
use super::db::Db;
use super::time;
use crate::error::AppResult;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct State {
    pub done: bool,
    pub snoozed_until: Option<DateTime<Utc>>,
}

pub async fn all(db: &Db) -> AppResult<HashMap<i64, State>> {
    let rows = db
        .api()
        .alert_states(client::AlertStatesInput {}, ())
        .await?;
    rows.into_iter()
        .map(|r| {
            let state = State {
                done: r.done,
                snoozed_until: time::get_opt(r.snoozed_until.as_deref())?,
            };
            Ok((int(&r.commitment_id)?, state))
        })
        .collect()
}

/// Joined or skipped: don't alert for this meeting again.
pub async fn done(db: &Db, id: i64) -> AppResult<()> {
    let input = client::AlertDoneInput {
        commitment: key(id),
    };
    Ok(db.api().alert_done(input, ()).await?)
}

pub async fn snooze(db: &Db, id: i64, until: DateTime<Utc>) -> AppResult<()> {
    let input = client::AlertSnoozeInput {
        commitment: key(id),
        until: time::put(until),
    };
    Ok(db.api().alert_snooze(input, ()).await?)
}
