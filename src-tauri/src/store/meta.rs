//! Small key-value facts about the app itself, e.g. when the scheduler last ran.

use chrono::{DateTime, Utc};

use super::client;
use super::convert::text_key;
use super::db::{Db, gone_ok};
use super::time;
use crate::error::{AppError, AppResult};

pub async fn time(db: &Db, key: &str) -> AppResult<Option<DateTime<Utc>>> {
    text(db, key)
        .await?
        .map(|stored| time::get(&stored))
        .transpose()
}

pub async fn set_time(db: &Db, key: &str, at: DateTime<Utc>) -> AppResult<()> {
    set_text(db, key, &time::put(at)).await
}

pub async fn text(db: &Db, key: &str) -> AppResult<Option<String>> {
    let input = client::MetaInput { key: text_key(key) };
    let row = db.api().meta(input, ()).await?;
    Ok(row.map(|r| r.value))
}

pub async fn set_text(db: &Db, key: &str, value: &str) -> AppResult<()> {
    let input = client::SetMetaInput {
        key: text_key(key),
        value: value.to_owned(),
    };
    Ok(db.api().set_meta(input, ()).await?)
}

pub async fn forget(db: &Db, key: &str) -> AppResult<()> {
    let input = client::ForgetMetaInput { key: text_key(key) };
    gone_ok(
        db.api()
            .forget_meta(input, ())
            .await
            .map_err(AppError::from),
    )
}
