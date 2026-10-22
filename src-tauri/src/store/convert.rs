//! Moves values between the generated client's types and the app's own: enums by their serde
//! names (e.g. `in_progress`), integer keys, and JSON columns.

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::client::Id;
use crate::error::{AppError, AppResult};

/// Converts between two types with the same serde form, e.g. the client's and the app's `Stage`.
pub fn same<A: Serialize, B: DeserializeOwned>(value: &A) -> AppResult<B> {
    Ok(serde_json::from_value(serde_json::to_value(value)?)?)
}

pub fn same_opt<A: Serialize, B: DeserializeOwned>(value: Option<&A>) -> AppResult<Option<B>> {
    value.map(same).transpose()
}

/// The key of a `serial` row.
pub fn int<E>(id: &Id<E>) -> AppResult<i64> {
    id.as_int()
        .ok_or_else(|| AppError::Db(format!("expected a number key, got {:?}", id.as_str())))
}

pub fn key<E>(id: i64) -> Id<E> {
    Id::from_int(id)
}

pub fn text_key<E>(id: &str) -> Id<E> {
    Id::from_raw(id)
}
