use crate::claude;
use crate::error::AppResult;

/// The one-off command that connects the user's own Claude Code sessions to Bridge.
#[tauri::command]
pub fn mcp_connect_command() -> AppResult<String> {
    claude::connect_command()
}
