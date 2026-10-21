//! Stores small enums as their serde names, e.g. `in_progress`.

use rusqlite::Row;
use rusqlite::types::Type;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::AppResult;

pub fn encode<T: Serialize>(value: &T) -> AppResult<String> {
    let json = serde_json::to_value(value)?;
    Ok(json
        .as_str()
        .map_or_else(|| json.to_string(), str::to_owned))
}

/// Reads an optional enum column written by [`encode`].
pub fn column<T: DeserializeOwned>(row: &Row<'_>, idx: usize) -> rusqlite::Result<Option<T>> {
    let text: Option<String> = row.get(idx)?;
    text.map(|t| {
        serde_json::from_value(serde_json::Value::String(t))
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, Box::new(e)))
    })
    .transpose()
}

/// Reads a required enum column written by [`encode`].
pub fn required<T: DeserializeOwned>(row: &Row<'_>, idx: usize) -> rusqlite::Result<T> {
    column(row, idx)?.ok_or(rusqlite::Error::InvalidColumnType(
        idx,
        String::new(),
        Type::Null,
    ))
}
