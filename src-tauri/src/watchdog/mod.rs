//! Keeps Claude moving on items an agent claimed (`claim_item`), without nagging: agents post
//! updates as they go, and only a quiet spell past `quiet_minutes` gets a check-in, at most `max_nudges` times before the
//! user is told instead.

mod decide;

pub use decide::held_by_claude;

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::claude::{self, Ask, Session};
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::item::{Stage, WorkItem};
use crate::domain::session::SessionRef;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::watch::{self, Watch};
use crate::store::{Db, events, items, runs, urgent};
use decide::{Action, Limits, decide, refreshed};

/// The last time anything was heard about an item: a state change or a history note.
pub async fn last_heard(db: &Db, item: &WorkItem) -> AppResult<DateTime<Utc>> {
    let latest = events::for_item(db, &item.id).await?.first().map(|e| e.at);
    Ok(latest.map_or(item.state_since, |at| at.max(item.state_since)))
}

/// Items a run is working on now.
async fn busy(app: &App) -> AppResult<Vec<String>> {
    Ok(runs::live(&app.db)
        .await?
        .into_iter()
        .map(|r| r.target)
        .collect())
}

/// Checks every item this machine's Claude holds. Runs from the scheduler tick.
pub async fn check(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    if !settings.watchdog.enabled {
        return Ok(());
    }
    let app = handle.state::<App>();
    let limits = Limits {
        quiet: Duration::minutes(settings.watchdog.quiet_minutes),
        max_nudges: settings.watchdog.max_nudges,
    };
    let now = Utc::now();
    let all = items::list(&app.db).await?;
    let watches = watch::all(&app.db).await?;
    // A watch lasts until the work is finished, so CI results keep reaching the session even
    // while the item sits with reviewers. Nudges only go to items Claude is carrying right now.
    for item in all
        .iter()
        .filter(|i| watches.contains_key(&i.id) && i.stage == Stage::Done)
    {
        watch::release(&app.db, &item.id).await?;
    }
    // Only claimed work is watched: checking in on items no agent took on would spend tokens on
    // runs that can't help. A run still working on the item isn't quiet, just busy.
    let running = busy(&app).await?;
    let held: Vec<&WorkItem> = all
        .iter()
        .filter(|i| {
            watches.contains_key(&i.id)
                && !running.contains(&i.id)
                && held_by_claude(i)
                && settings.node_for(&i.area) == settings.node
        })
        .collect();
    for item in held {
        let heard = last_heard(&app.db, item).await?;
        let current = refreshed(&watches.get(&item.id).cloned().unwrap_or_default(), heard);
        match decide(heard, &current, now, limits) {
            Action::Wait => {}
            Action::Nudge => nudge(handle, item, &current, now - heard).await?,
            Action::Escalate => escalate(handle, item, &current).await?,
        }
    }
    Ok(())
}

async fn record(handle: &AppHandle, item: &WorkItem, w: &Watch) -> AppResult<()> {
    watch::record(&handle.state::<App>().db, &item.id, w).await
}

async fn nudge(handle: &AppHandle, item: &WorkItem, w: &Watch, quiet: Duration) -> AppResult<()> {
    // A build that may not run Claude has nobody to nudge; the item keeps its count.
    let Ok(command) = handle
        .state::<App>()
        .settings()?
        .claude()
        .map(str::to_owned)
    else {
        return Ok(());
    };
    record(
        handle,
        item,
        &Watch {
            nudges: w.nudges + 1,
            last_nudge: Some(Utc::now()),
            ..w.clone()
        },
    )
    .await?;
    let app = handle.state::<App>();
    let quiet_text = format!("{} minutes", quiet.num_minutes());
    let vars = [
        ("item", item.id.as_str()),
        ("title", item.title.as_str()),
        ("label", item.state_label.as_str()),
        ("quiet", quiet_text.as_str()),
    ];
    let prompt = render(&app.config, Prompt::CheckIn, &vars)?;
    check_in(command, prompt, w.session.clone());
    Ok(())
}

/// Asks the item's session (or a fresh one) to report in, in the background.
fn check_in(command: String, prompt: String, session: Option<SessionRef>) {
    tauri::async_runtime::spawn(async move {
        // A failed check-in counts as a nudge; the next one, or the escalation, follows anyway.
        drop(
            claude::ask_with(
                &command,
                &prompt,
                &Ask {
                    tools: &[],
                    session: session
                        .as_ref()
                        .map_or(Session::Fresh, |s| Session::Resume(&s.id)),
                    cwd: session.as_ref().and_then(|s| s.cwd.as_deref()),
                    permission_mode: None,
                    run: None,
                },
            )
            .await,
        );
    });
}

async fn escalate(handle: &AppHandle, item: &WorkItem, w: &Watch) -> AppResult<()> {
    record(
        handle,
        item,
        &Watch {
            nudges: w.nudges + 1,
            last_nudge: Some(Utc::now()),
            ..w.clone()
        },
    )
    .await?;
    let flag = Urgent {
        id: 0,
        title: format!("Claude has gone quiet on {}", item.id),
        detail: format!("{} · no update after {} check-ins", item.title, w.nudges),
        area: item.area.clone(),
        raised_at: Utc::now(),
        schedule_id: None,
    };
    urgent::raise(&handle.state::<App>().db, &flag).await?;
    drop(crate::notify::send(handle, &flag.title, &flag.detail));
    Ok(())
}
