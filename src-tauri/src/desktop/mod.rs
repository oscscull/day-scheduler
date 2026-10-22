//! Keeping Quarterdeck running quietly: a tray icon, closing to the dock or taskbar, and launching
//! at login.
//! The scheduler and meeting alerts only work while the app is open.

pub mod login;
pub mod tray;

use tauri::{AppHandle, Manager as _, Window, WindowEvent};

pub const MAIN: &str = "main";

/// Brings the main window back, e.g. from the tray or a second launch.
pub fn show_main(handle: &AppHandle) {
    if let Some(window) = handle.get_webview_window(MAIN) {
        // A failure here leaves the app running; the next click tries again.
        drop(window.unminimize());
        drop(window.show());
        drop(window.set_focus());
    }
}

/// Closing the main window minimizes it instead of quitting, so the dock or taskbar brings it back
/// on every platform. Coming back to it syncs at once.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() != MAIN {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        drop(window.minimize());
    }
    if matches!(event, WindowEvent::Focused(true)) {
        // The regular poll still runs if this fails.
        drop(crate::scheduler::sync_on_focus(window.app_handle()));
    }
}
