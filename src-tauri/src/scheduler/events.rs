//! Starts event schedules for things connectors noticed.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use super::tick::start;
use crate::app::App;
use crate::config::{pause, schedules};
use crate::connectors::github::Fired;
use crate::domain::schedule::{EventKind, Trigger};
use crate::error::AppResult;

/// A schedule reacts to events in its own area, on the machine that runs that area.
pub async fn start_for(handle: &AppHandle, fired: &[Fired]) -> AppResult<()> {
    let app = handle.state::<App>();
    if fired.is_empty() || pause::current(&app.config, Utc::now())?.is_some() {
        return Ok(());
    }
    let settings = app.settings()?;
    let all = schedules::list(&app.config)?.schedules;
    for event in fired {
        let matching: Vec<_> = all
            .iter()
            .filter(|s| {
                s.enabled
                    && s.trigger == Trigger::Event { kind: event.kind }
                    && s.area == event.item_area
                    && settings.node_for(&s.area) == settings.node
            })
            .collect();
        for schedule in &matching {
            start(handle, (*schedule).clone(), Some(event.trigger.clone()));
        }
        if matches!(event.kind, EventKind::CiFailed | EventKind::CiPassed)
            && settings.node_for(&event.item_area) == settings.node
        {
            super::ci::react(handle, &settings, event, !matching.is_empty()).await?;
        }
    }
    Ok(())
}
