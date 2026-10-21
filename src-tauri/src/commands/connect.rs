use crate::claude::connect::{self, Connection};
use crate::error::AppResult;

/// Whether the user's own Claude Code sessions are connected to Bridge, and how to connect them.
#[tauri::command]
pub fn claude_connection() -> AppResult<Connection> {
    connect::status()
}
