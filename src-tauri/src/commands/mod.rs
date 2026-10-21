//! Thin Tauri command handlers. Each one loads, delegates, and returns.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands must take their arguments and State by value"
)]
#![allow(
    clippy::let_underscore_must_use,
    reason = "tauri::command's async expansion discards a value; not our code"
)]

pub mod alerts;
pub mod areas;
pub mod chat;
pub mod commitments;
pub mod connect;
pub mod items;
pub mod schedules;
pub mod screens;
pub mod todos;
pub mod urgent;

/// Every command the frontend can call.
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        areas::areas,
        alerts::due_alert,
        alerts::alert_done,
        alerts::alert_snooze,
        connect::mcp_connect_command,
        screens::today,
        commitments::add_commitment,
        commitments::delete_commitment,
        commitments::hide_commitment,
        todos::add_todo,
        todos::set_todo_done,
        screens::work_items,
        screens::agent,
        items::reorder_items,
        items::save_item,
        items::delete_item,
        items::add_ask,
        items::resolve_ask,
        items::item_detail,
        schedules::set_schedule_enabled,
        schedules::save_schedule,
        schedules::delete_schedule,
        schedules::explain_cron,
        schedules::run_schedule_now,
        schedules::pause_all,
        schedules::resume_all,
        urgent::dismiss_urgent,
        urgent::dismiss_all_urgent,
        urgent::resolve_urgent,
        chat::ask_claude,
        chat::draft_schedule,
    ]
}
