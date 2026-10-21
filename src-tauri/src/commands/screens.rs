use chrono::Utc;
use tauri::State;

use crate::app::App;
use crate::config::schedules;
use crate::domain::item::WorkItem;
use crate::error::AppResult;
use crate::store::items;
use crate::views::agent::{self, AgentView};
use crate::views::today::{self, TodayView};

#[tauri::command]
pub fn today(app: State<'_, App>) -> AppResult<TodayView> {
    let settings = app.settings()?;
    app.expire(&settings)?;
    app.db
        .with(|conn| today::build(conn, settings.priority_limit, Utc::now()))
}

/// All work items in priority order. Board, In flight and Priorities share this.
#[tauri::command]
pub fn work_items(app: State<'_, App>) -> AppResult<Vec<WorkItem>> {
    app.db.with(items::list)
}

#[tauri::command]
pub fn agent(app: State<'_, App>) -> AppResult<AgentView> {
    let settings = app.settings()?;
    app.expire(&settings)?;
    let loaded = schedules::list(&app.config)?;
    app.db.with(|conn| agent::build(conn, loaded))
}
