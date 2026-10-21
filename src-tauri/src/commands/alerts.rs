use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _, State};

use crate::alerts;
use crate::app::App;
use crate::domain::commitment::Commitment;
use crate::error::AppResult;
use crate::store::alert_state;

const SNOOZE: Duration = Duration::minutes(1);

/// The meeting the alert window should show.
#[tauri::command]
pub fn due_alert(app: State<'_, App>) -> AppResult<Option<Commitment>> {
    alerts::current(&app)
}

/// Joined or skipped: this meeting won't alert again.
#[tauri::command]
pub fn alert_done(handle: AppHandle, id: i64) -> AppResult<()> {
    handle
        .state::<App>()
        .db
        .with(|conn| alert_state::done(conn, id))?;
    alerts::close(&handle)
}

#[tauri::command]
pub fn alert_snooze(handle: AppHandle, id: i64) -> AppResult<()> {
    handle
        .state::<App>()
        .db
        .with(|conn| alert_state::snooze(conn, id, Utc::now() + SNOOZE))?;
    alerts::close(&handle)
}
