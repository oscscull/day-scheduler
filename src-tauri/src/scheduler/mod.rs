//! Fires cron schedules on time on this machine, and records the ones that were missed.

mod due;
mod tick;

pub use tick::start as run_now;

use std::time::Duration;

use tauri::AppHandle;

const EVERY: Duration = Duration::from_secs(30);

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
