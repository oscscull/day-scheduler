//! Keeping Quarterdeck running quietly: a menu-bar icon, closing to it, and launching at login.
//! The scheduler and meeting alerts only work while the app is open.

pub mod login;
pub mod tray;

use tauri::{AppHandle, Manager as _, RunEvent, Window, WindowEvent};

pub const MAIN: &str = "main";

/// Brings the main window back from hidden.
pub fn show_main(handle: &AppHandle) {
    if let Some(window) = handle.get_webview_window(MAIN) {
        // A failure here leaves the app running; the next click tries again.
        drop(window.show());
        drop(window.set_focus());
    }
}

/// Closing the main window hides it instead of quitting. Coming back to it syncs at once.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        drop(window.hide());
    }
    if matches!(event, WindowEvent::Focused(true)) {
        // The regular poll still runs if this fails.
        drop(crate::scheduler::sync_on_focus(window.app_handle()));
    }
}

/// Clicking the dock icon reopens the window.
pub fn on_run_event(handle: &AppHandle, event: &RunEvent) {
    if let RunEvent::Reopen { .. } = event {
        show_main(handle);
    }
}
