//! Bridge: tracks work in flight and runs recurring Claude jobs.

mod alerts;
mod app;
mod claude;
mod cli;
mod commands;
mod config;
mod connectors;
mod cron;
mod desktop;
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

fn settings_for_login(config: &config::Config) -> bool {
    config::settings::load(config).map_or(true, |s| s.launch_at_login)
}

/// Opens the database and config folder, then hands both to every command.
fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let db = store::Db::open(&paths::data_dir()?)?;
    let config = config::Config::open(paths::config_dir()?)?;
    let launch_at_login = settings_for_login(&config);
    app.manage(app::App { db, config });
    desktop::tray::build(app.handle())?;
    // Launch-at-login is a convenience; failing to set it must not stop the app opening.
    drop(desktop::login::apply(app.handle(), launch_at_login));
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .on_window_event(desktop::on_window_event)
        .setup(setup)
        .invoke_handler(commands::handler())
        .build(tauri::generate_context!())?
        .run(|handle, event| desktop::on_run_event(handle, &event));
    Ok(())
}
