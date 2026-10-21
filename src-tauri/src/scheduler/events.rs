//! Starts event schedules for things connectors noticed.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use super::tick::start;
use crate::app::App;
use crate::config::{pause, schedules};
use crate::connectors::github::Fired;
use crate::domain::schedule::Trigger;
use crate::error::AppResult;

/// A schedule reacts to events in its own area, on the machine that runs that area.
pub fn start_for(handle: &AppHandle, fired: &[Fired]) -> AppResult<()> {
    let app = handle.state::<App>();
    if fired.is_empty() || pause::current(&app.config, Utc::now())?.is_some() {
        return Ok(());
    }
    let settings = app.settings()?;
    let all = schedules::list(&app.config)?.schedules;
    for event in fired {
        let matching = all.iter().filter(|s| {
            s.enabled
                && s.trigger == Trigger::Event { kind: event.kind }
                && s.area == event.item_area
                && settings.node_for(&s.area) == settings.node
        });
        for schedule in matching {
            start(handle, schedule.clone(), Some(event.trigger.clone()));
        }
    }
    Ok(())
}
