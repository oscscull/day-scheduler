//! One pass of the scheduler: take inbox files, then start or mark missed any due schedules.

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Manager as _};

use super::due::{Plan, plan};
use crate::app::App;
use crate::claude::runner;
use crate::config::settings::Settings;
use crate::config::{pause, schedules};
use crate::domain::run::{Outcome, Run};
use crate::domain::schedule::{Schedule, Trigger};
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{meta, run_requests, runs, urgent};

const LAST_TICK: &str = "scheduler.last_tick";
/// A run this late still goes ahead; anything older counts as missed.
const GRACE: Duration = Duration::minutes(10);

/// The stretch of time since the previous tick.
#[derive(Debug, Clone, Copy)]
struct Window {
    last: DateTime<Utc>,
    now: DateTime<Utc>,
}

pub fn tick(handle: &AppHandle) -> AppResult<()> {
    let app = handle.state::<App>();
    let settings = app.settings()?;
    let raised = app.take_inbox(&settings)?;
    if raised > 0 {
        // Notifications are a courtesy; a failure must not stop the scheduler.
        drop(notify::send(
            handle,
            "Bridge",
            &format!("{raised} new urgent item(s)"),
        ));
    }
    start_requested(handle)?;
    crate::alerts::check(handle)?;
    let urgent_count = app.db.with(urgent::list)?.len();
    crate::desktop::tray::show_urgent(handle, urgent_count)?;
    super::connectors::sync_if_due(handle, &settings)?;
    let now = Utc::now();
    let last = app.db.with(|c| meta::time(c, LAST_TICK))?;
    app.db.with(|c| meta::set_time(c, LAST_TICK, now))?;
    // First launch has no history to catch up on; a pause means nothing runs or is missed.
    let Some(last) = last else { return Ok(()) };
    if pause::current(&app.config, now)?.is_some() {
        return Ok(());
    }
    for schedule in schedules::list(&app.config)?.schedules {
        if schedule.enabled && settings.node_for(&schedule.area) == settings.node {
            consider(handle, &settings, &schedule, Window { last, now })?;
        }
    }
    Ok(())
}

fn consider(
    handle: &AppHandle,
    settings: &Settings,
    schedule: &Schedule,
    w: Window,
) -> AppResult<()> {
    let Trigger::Cron { expr } = &schedule.trigger else {
        return Ok(());
    };
    // A schedule with a bad expression is reported on the Agent tab instead.
    let Ok(p) = plan(expr, w.last, w.now, GRACE) else {
        return Ok(());
    };
    if p.missed.is_some() {
        record_missed(handle, schedule, settings.node_for(&schedule.area), &p)?;
    }
    if p.run && !is_running(handle, &schedule.id)? {
        start(handle, schedule.clone(), None);
    }
    Ok(())
}

/// Starts schedules an agent asked to run now. These run even while paused: someone asked.
fn start_requested(handle: &AppHandle) -> AppResult<()> {
    let app = handle.state::<App>();
    for id in app.db.with(run_requests::take_all)? {
        if let Ok(schedule) = schedules::find(&app.config, &id)
            && !is_running(handle, &id)?
        {
            start(handle, schedule, None);
        }
    }
    Ok(())
}

fn is_running(handle: &AppHandle, id: &str) -> AppResult<bool> {
    let live = handle.state::<App>().db.with(runs::live)?;
    Ok(live.iter().any(|r| r.schedule_id.as_deref() == Some(id)))
}

/// Runs a schedule in the background; the runner records its own outcome.
pub fn start(handle: &AppHandle, schedule: Schedule, trigger: Option<String>) {
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        drop(runner::run(&handle, &schedule, trigger.as_deref()).await);
    });
}

/// Records the latest missed occurrence as a run and raises it as urgent.
fn record_missed(handle: &AppHandle, schedule: &Schedule, node: &str, p: &Plan) -> AppResult<()> {
    let Some(at) = p.missed else { return Ok(()) };
    let count = p.missed_count;
    let reason = "Bridge wasn't running, or the machine was asleep".to_owned();
    let run = Run {
        id: 0,
        schedule_id: Some(schedule.id.clone()),
        what: schedule.name.clone(),
        target: String::new(),
        node: node.to_owned(),
        started_at: at,
        finished_at: Some(at),
        outcome: Some(Outcome::Missed),
        summary: Some(reason.clone()),
    };
    let times = if count > 1 {
        format!(" ({count} times)")
    } else {
        String::new()
    };
    let item = Urgent {
        id: 0,
        title: format!("{} didn't run{times}", schedule.name),
        detail: format!("Due on {node}. {reason}."),
        area: schedule.area.clone(),
        raised_at: Utc::now(),
        schedule_id: Some(schedule.id.clone()),
    };
    handle.state::<App>().db.with(|c| {
        runs::insert(c, &run)?;
        urgent::raise(c, &item)
    })?;
    drop(notify::send(handle, &item.title, &item.detail));
    Ok(())
}
