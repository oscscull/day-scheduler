//! Fires cron schedules on time on this machine, and records the ones that were missed.

mod connectors;
mod due;
mod events;
mod tick;

use crate::domain::schedule::Schedule;

use std::time::Duration;

use tauri::AppHandle;

const EVERY: Duration = Duration::from_secs(30);

/// Runs a schedule now, in the background.
pub fn run_now(handle: &AppHandle, schedule: Schedule) {
    tick::start(handle, schedule, None);
}

/// Starts the scheduler loop for as long as the app runs.
pub fn spawn(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // One bad tick (e.g. a half-written settings file) must not stop the loop.
            drop(tick::tick(&handle));
            tokio::time::sleep(EVERY).await;
        }
    });
}
