//! Where Quarterdeck keeps things. Shared by the app and the MCP server so both see the same data.

use std::path::PathBuf;

use crate::error::{AppError, AppResult};

/// Must match `identifier` in `tauri.conf.json`.
const APP_ID: &str = "com.oscscull.quarterdeck";

pub fn home() -> AppResult<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::NotFound("the HOME folder".to_owned()))
}

/// The database lives here.
pub fn data_dir() -> AppResult<PathBuf> {
    Ok(home()?.join("Library/Application Support").join(APP_ID))
}

/// Behaviour the user and agents edit: settings, prompts, schedules, inbox.
pub fn config_dir() -> AppResult<PathBuf> {
    Ok(home()?.join(".config/quarterdeck"))
}

/// Claude Code's user config, where `claude mcp add --scope user` registers servers.
pub fn claude_config() -> AppResult<PathBuf> {
    Ok(home()?.join(".claude.json"))
}
