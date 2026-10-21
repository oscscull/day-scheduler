//! The meeting alert: a small window that floats on top with a countdown and a chime.

mod due;

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager as _, WebviewUrl, WebviewWindowBuilder};

use crate::app::App;
use crate::domain::commitment::Commitment;
use crate::error::AppResult;
use crate::store::{alert_state, commitments};

pub const WINDOW: &str = "alert";

/// The meeting whose alert should be showing now, if any.
pub fn current(app: &App) -> AppResult<Option<Commitment>> {
    let settings = app.settings()?.alerts;
    if !settings.enabled {
        return Ok(None);
    }
    let window = due::Window {
        lead: Duration::minutes(settings.lead_minutes),
        linger: Duration::minutes(settings.linger_minutes),
    };
    app.db.with(|conn| {
        let meetings = commitments::list(conn)?;
        let states = alert_state::all(conn)?;
        Ok(due::due(&meetings, &states, Utc::now(), window).cloned())
    })
}

/// Opens the alert window when a meeting is close, and closes it when none is.
pub fn check(handle: &AppHandle) -> AppResult<()> {
    let showing = handle.get_webview_window(WINDOW);
    match (current(&handle.state::<App>())?, showing) {
        (Some(_), None) => open(handle),
        (None, Some(_)) => close(handle),
        (Some(_) | None, _) => Ok(()),
    }
}

fn open(handle: &AppHandle) -> AppResult<()> {
    WebviewWindowBuilder::new(handle, WINDOW, WebviewUrl::App("index.html".into()))
        .title("Bridge")
        .inner_size(440.0, 280.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .center()
        .focused(true)
        .build()?;
    Ok(())
}

/// Closes the alert after Join, Snooze or Skip; the next check reopens it if another is due.
/// macOS only lets the main thread change windows, and the scheduler runs on a worker.
pub fn close(handle: &AppHandle) -> AppResult<()> {
    let main = handle.clone();
    handle.run_on_main_thread(move || {
        if let Some(window) = main.get_webview_window(WINDOW) {
            // Nothing useful to do if it's already gone.
            drop(window.destroy());
        }
    })?;
    Ok(())
}
