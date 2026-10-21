//! Quiet native notifications: runs finishing, things turning urgent.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt as _;

use crate::error::{AppError, AppResult};

pub fn send(handle: &AppHandle, title: &str, body: &str) -> AppResult<()> {
    handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| AppError::Notify(e.to_string()))
}
