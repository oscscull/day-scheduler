// Prevents an extra console window on Windows in release. Do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("mcp") => return day_scheduler_lib::serve_mcp(),
        Some("serve") => return day_scheduler_lib::serve_peers(),
        _ => {}
    }
    match day_scheduler_lib::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
