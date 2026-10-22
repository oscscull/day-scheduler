//! Fires cron schedules on time on this machine, and records the ones that were missed.

mod answers;
mod ci;
mod connectors;
mod dispatch;
mod due;
mod events;
mod focus;
mod handoff;
mod queue;
mod tick;
mod undelivered;

use crate::domain::schedule::Schedule;

use std::time::Duration;

use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::store::meta;

const LAST_ERROR: &str = "scheduler.last_error";

const EVERY: Duration = Duration::from_secs(30);

/// Runs a schedule now, in the background.
pub use focus::sync_on_focus;

pub fn run_now(handle: &AppHandle, schedule: Schedule) {
    tick::start(handle, schedule, None);
}

/// Starts the scheduler loop for as long as the app runs.
pub fn spawn(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // One bad tick (e.g. a half-written settings file) must not stop the loop; the error is
            // kept for diagnostics instead.
            let error = tick::tick(&handle)
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default();
            drop(
                handle
                    .state::<App>()
                    .db
                    .with(|c| meta::set_text(c, LAST_ERROR, &error)),
            );
            tokio::time::sleep(EVERY).await;
        }
    });
}
