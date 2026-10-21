use tauri::State;

use crate::app::App;
use crate::config::settings::Area;
use crate::error::AppResult;

/// The areas from `bridge.toml`, for the switcher and colour dots.
#[tauri::command]
pub fn areas(app: State<'_, App>) -> AppResult<Vec<Area>> {
    Ok(app.settings()?.areas)
}
