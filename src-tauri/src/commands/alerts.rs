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
pub async fn due_alert(app: State<'_, App>) -> AppResult<Option<Commitment>> {
    alerts::current(&app).await
}

/// Joined or skipped: this meeting won't alert again.
#[tauri::command]
pub async fn alert_done(handle: AppHandle, id: i64) -> AppResult<()> {
    alert_state::done(&handle.state::<App>().db, id).await?;
    alerts::close(&handle)
}

#[tauri::command]
pub async fn alert_snooze(handle: AppHandle, id: i64) -> AppResult<()> {
    alert_state::snooze(&handle.state::<App>().db, id, Utc::now() + SNOOZE).await?;
    alerts::close(&handle)
}
