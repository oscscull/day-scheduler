use chrono::Local;
use tauri::State;

use crate::app::App;
use crate::config::schedules;
use crate::cron::explain::{Explanation, explain};
use crate::domain::schedule::Schedule;
use crate::error::AppResult;
use crate::store::runs;

const PREVIEW_RUNS: usize = 3;

#[tauri::command]
pub fn set_schedule_enabled(app: State<'_, App>, id: String, enabled: bool) -> AppResult<()> {
    let mut schedule = schedules::find(&app.config, &id)?;
    schedule.enabled = enabled;
    schedules::save(&app.config, &schedule)
}

/// Deletes a schedule file and its run history.
#[tauri::command]
pub fn delete_schedule(app: State<'_, App>, id: String) -> AppResult<()> {
    schedules::delete(&app.config, &id)?;
    app.db.with(|conn| runs::delete_for(conn, &id))
}

/// Writes a schedule file. An empty id creates a new one.
#[tauri::command]
pub fn save_schedule(app: State<'_, App>, mut schedule: Schedule) -> AppResult<String> {
    if schedule.id.is_empty() {
        schedule.id = schedules::free_id(&app.config, &schedules::slug(&schedule.name));
    }
    schedules::save(&app.config, &schedule)?;
    Ok(schedule.id)
}

/// Live explanation while the user edits a cron expression.
#[tauri::command]
pub fn explain_cron(expr: String) -> AppResult<Explanation> {
    explain(&expr, Local::now(), PREVIEW_RUNS)
}
