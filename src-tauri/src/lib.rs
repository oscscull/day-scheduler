//! Bridge: tracks work in flight and runs recurring Claude jobs.

mod app;
mod claude;
mod commands;
mod config;
mod cron;
mod domain;
mod error;
mod mcp;
mod notify;
mod paths;
mod scheduler;
mod store;
mod today;
mod views;
mod work;

use tauri::Manager as _;

/// Opens the database and config folder, then hands both to every command.
fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let db = store::Db::open(&paths::data_dir()?)?;
    let config = config::Config::open(paths::config_dir()?)?;
    app.manage(app::App { db, config });
    scheduler::spawn(app.handle().clone());
    Ok(())
}

/// Serves the Model Context Protocol over stdio instead of opening the app. Started as
/// `<binary> mcp`, so any Claude session can read and change Bridge.
#[must_use]
pub fn serve_mcp() -> std::process::ExitCode {
    mcp::serve()
}

/// Builds and runs the Tauri application.
///
/// # Errors
/// Returns an error if the Tauri runtime fails to start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(
    clippy::exit,
    reason = "generate_context! expands to code calling process::exit; not ours"
)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::areas::areas,
            commands::connect::mcp_connect_command,
            commands::screens::today,
            commands::commitments::add_commitment,
            commands::commitments::delete_commitment,
            commands::todos::add_todo,
            commands::todos::set_todo_done,
            commands::screens::work_items,
            commands::screens::agent,
            commands::items::reorder_items,
            commands::items::save_item,
            commands::items::delete_item,
            commands::items::add_ask,
            commands::items::resolve_ask,
            commands::items::item_detail,
            commands::schedules::set_schedule_enabled,
            commands::schedules::save_schedule,
            commands::schedules::delete_schedule,
            commands::schedules::explain_cron,
            commands::schedules::run_schedule_now,
            commands::schedules::pause_all,
            commands::schedules::resume_all,
            commands::urgent::dismiss_urgent,
            commands::urgent::dismiss_all_urgent,
            commands::urgent::resolve_urgent,
            commands::chat::ask_claude,
            commands::chat::draft_schedule,
        ])
        .run(tauri::generate_context!())
}
