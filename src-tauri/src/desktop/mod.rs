//! Keeping Bridge running quietly: a menu-bar icon, closing to it, and launching at login.
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

/// Closing the main window hides it instead of quitting.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event
        && window.label() == MAIN
    {
        api.prevent_close();
        drop(window.hide());
    }
}

/// Clicking the dock icon reopens the window.
pub fn on_run_event(handle: &AppHandle, event: &RunEvent) {
    if let RunEvent::Reopen { .. } = event {
        show_main(handle);
    }
}
