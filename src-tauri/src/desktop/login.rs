use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt as _;

use crate::error::{AppError, AppResult};

/// Matches launch-at-login to the setting. Dev builds never register, so a debug binary isn't
/// left starting at login.
pub fn apply(handle: &AppHandle, wanted: bool) -> AppResult<()> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    let launcher = handle.autolaunch();
    let current = launcher
        .is_enabled()
        .map_err(|e| AppError::Desktop(e.to_string()))?;
    let result = match (wanted, current) {
        (true, false) => launcher.enable(),
        (false, true) => launcher.disable(),
        (true, true) | (false, false) => Ok(()),
    };
    result.map_err(|e| AppError::Desktop(e.to_string()))
}
