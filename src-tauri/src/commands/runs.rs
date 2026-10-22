use tauri::State;

use crate::app::App;
use crate::domain::step::Step;
use crate::error::AppResult;
use crate::runlog;
use crate::views::live::{self, LiveRun};

#[tauri::command]
pub async fn live_runs(app: State<'_, App>) -> AppResult<Vec<LiveRun>> {
    live::build(&app.db).await
}

#[tauri::command]
pub fn run_log(id: i64) -> AppResult<Vec<Step>> {
    runlog::read(&runlog::dir()?, id)
}
