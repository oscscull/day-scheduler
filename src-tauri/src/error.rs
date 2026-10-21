use serde::{Serialize, Serializer};

/// Every error the backend can return to the frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("invalid cron expression: {0}")]
    Cron(#[from] croner::errors::CronError),
    #[error("bad data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("bad settings file: {0}")]
    TomlRead(#[from] toml::de::Error),
    #[error("could not write settings: {0}")]
    TomlWrite(#[from] toml::ser::Error),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("app error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("claude failed: {0}")]
    Claude(String),
    #[error("{0}")]
    Connector(String),
    #[error("{0}")]
    Desktop(String),
    #[error("couldn't notify: {0}")]
    Notify(String),
    #[error("{0}")]
    BadInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("the database lock was poisoned")]
    Lock,
}

/// Errors cross the IPC boundary as plain messages.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
