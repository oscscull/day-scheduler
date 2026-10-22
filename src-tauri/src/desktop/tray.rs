//! The menu-bar icon. It shows the urgent count only when something is urgent.

use chrono::{Duration, Local, Utc};
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager as _};

use super::show_main;
use crate::app::App;
use crate::config::pause;
use crate::error::AppResult;

const ID: &str = "quarterdeck";
const OPEN: &str = "open";
const PAUSE_TODAY: &str = "pause-today";
const RESUME: &str = "resume";
const QUIT: &str = "quit";

pub fn build(handle: &AppHandle) -> AppResult<TrayIcon> {
    let menu = Menu::with_items(
        handle,
        &[
            &MenuItem::with_id(handle, OPEN, "Open Quarterdeck", true, None::<&str>)?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(
                handle,
                PAUSE_TODAY,
                "Pause all for today",
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(handle, RESUME, "Resume all", true, None::<&str>)?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItem::with_id(handle, QUIT, "Quit Quarterdeck", true, None::<&str>)?,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id(ID)
        .menu(&menu)
        .tooltip(crate::profile::NAME)
        .on_menu_event(|h, e| on_menu(h, &e));
    if let Some(icon) = handle.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    Ok(builder.build(handle)?)
}

fn end_of_today() -> chrono::DateTime<Utc> {
    let tomorrow = Local::now().date_naive() + Duration::days(1);
    tomorrow
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Local).single())
        .map_or_else(
            || Utc::now() + Duration::hours(12),
            |t| t.with_timezone(&Utc),
        )
}

fn on_menu(handle: &AppHandle, event: &MenuEvent) {
    let config = &handle.state::<App>().config;
    // Menu actions have nowhere to report errors; the Agent tab shows the pause state.
    match event.id().as_ref() {
        OPEN => show_main(handle),
        PAUSE_TODAY => drop(pause::pause(config, Some(end_of_today()))),
        RESUME => drop(pause::resume(config)),
        QUIT => handle.exit(0),
        _ => {}
    }
}

/// Puts the urgent count next to the icon, or nothing when all is calm.
pub fn show_urgent(handle: &AppHandle, count: usize) -> AppResult<()> {
    if let Some(tray) = handle.tray_by_id(ID) {
        let title = (count > 0).then(|| format!("{count} urgent"));
        tray.set_title(title)?;
    }
    Ok(())
}
