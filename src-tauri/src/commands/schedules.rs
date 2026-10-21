use chrono::{DateTime, Local, Utc};
use tauri::{AppHandle, Manager as _, State};

use crate::app::App;
use crate::config::{pause, schedules};
use crate::cron::explain::{Explanation, explain};
use crate::domain::schedule::Schedule;
use crate::error::{AppError, AppResult};
use crate::peers;
use crate::scheduler;
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
pub async fn save_schedule(app: State<'_, App>, mut schedule: Schedule) -> AppResult<String> {
    if schedule.id.is_empty() {
        if let Some(id) =
            peers::handoff::create_elsewhere_async(app.settings()?, schedule.clone()).await?
        {
            return Ok(id);
        }
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

/// Starts a schedule in the background; the Agent view shows it as live until it finishes.
#[tauri::command]
pub fn run_schedule_now(handle: AppHandle, id: String) -> AppResult<()> {
    let schedule = schedules::find(&handle.state::<App>().config, &id)?;
    scheduler::run_now(&handle, schedule);
    Ok(())
}

/// Pauses every schedule until `until` (an RFC 3339 time), or until resumed if `None`.
#[tauri::command]
pub fn pause_all(app: State<'_, App>, until: Option<String>) -> AppResult<()> {
    let until = until
        .map(|text| DateTime::parse_from_rfc3339(&text).map(|t| t.with_timezone(&Utc)))
        .transpose()
        .map_err(|e| AppError::NotFound(format!("a valid pause end time ({e})")))?;
    pause::pause(&app.config, until)
}

#[tauri::command]
pub fn resume_all(app: State<'_, App>) -> AppResult<()> {
    pause::resume(&app.config)
}
