//! Quarterdeck: tracks work in flight and runs recurring Claude jobs.

mod alerts;
mod answer;
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
mod peers;
mod priorities;
mod rename;
mod scheduler;
mod store;
mod today;
mod views;
mod watchdog;
mod work;
mod workspace;

use tauri::Manager as _;

/// Starts serving peers when `[sync]` is enabled, making a token the first time.
fn start_sync(config: &config::Config) -> error::AppResult<()> {
    let sync = config::settings::load(config)?.sync;
    if !sync.enabled {
        return Ok(());
    }
    start_token(config)?;
    peers::server::spawn(sync.listen())
}

/// Makes and saves the sync token the first time it's needed.
fn start_token(config: &config::Config) -> error::AppResult<()> {
    if config::settings::load(config)?.sync.token.is_empty() {
        config::edit::set_sync_token(config, &peers::new_token()?)?;
    }
    Ok(())
}

fn settings_for_login(config: &config::Config) -> bool {
    config::settings::load(config).map_or(true, |s| s.launch_at_login)
}

/// Opens the database and config folder, then hands both to every command.
fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let db = store::Db::open(&paths::data_dir()?)?;
    // Only the app runs Claude, so anything still marked running is from before this start.
    db.with(|c| store::runs::close_interrupted(c, chrono::Utc::now()))?;
    let config = config::Config::open(paths::config_dir()?)?;
    let launch_at_login = settings_for_login(&config);
    // Serving peers is optional; a busy port must not stop the app opening.
    drop(start_sync(&config));
    app.manage(app::App { db, config });
    desktop::tray::build(app.handle())?;
    // Launch-at-login is a convenience; failing to set it must not stop the app opening.
    drop(desktop::login::apply(app.handle(), launch_at_login));
    scheduler::spawn(app.handle().clone());
    Ok(())
}

/// Gets the machine ready before any entry point runs: moves an install from the old name.
/// False if that failed, in which case nothing should open the config or database.
#[must_use]
pub fn prepare() -> bool {
    rename::migrate().is_ok()
}

/// Serves the Model Context Protocol over stdio instead of opening the app. Started as
/// `<binary> mcp`, so any Claude session can read and change Quarterdeck.
#[must_use]
pub fn serve_mcp() -> std::process::ExitCode {
    mcp::serve()
}

/// Serves this machine's Quarterdeck to peers in the foreground, without the app window. Started as
/// `<binary> serve`, e.g. on a machine without a screen.
#[must_use]
pub fn serve_peers() -> std::process::ExitCode {
    let started = paths::config_dir()
        .and_then(config::Config::open)
        .and_then(|config| {
            start_token(&config)?;
            peers::server::run(config::settings::load(&config)?.sync.listen())
        });
    if started.is_ok() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
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
