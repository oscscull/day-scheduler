//! After Claude finishes in a worktree, Meatpuppet does the GitHub side: it pushes the commits and,
//! for an issue, opens the PR and makes the item that PR's. Nothing is published while a question
//! on the item is open, since the work is waiting on the user.

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::connectors::github::{self, OpenedPr};
use crate::domain::event::Change;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{asks, events, items, urgent};
use crate::workspace::{self, Published, Workspace};

pub async fn after_run(handle: &AppHandle, item_id: &str, ws: &Workspace) {
    let app = handle.state::<App>();
    let waiting = app
        .db
        .with(|c| asks::for_item(c, item_id))
        .map_or(true, |open| !open.is_empty());
    if waiting {
        return;
    }
    let published = workspace::publish(ws).await;
    let failed = published.as_ref().err().map(ToString::to_string);
    let recorded = app
        .db
        .with(|c| record(c, (item_id, ws), published, Utc::now()));
    let problem = recorded.err().map(|e| e.to_string()).or(failed);
    if let Some(problem) = problem {
        // The urgent item and history note carry it; the notification is a courtesy.
        let title = format!("Couldn't publish Claude's work on {item_id}");
        drop(notify::send(handle, &title, &problem));
    }
}

fn record(
    conn: &Connection,
    (item_id, ws): (&str, &Workspace),
    published: AppResult<Published>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    match published {
        Ok(Published::Nothing) => Ok(()),
        Ok(Published::Pushed) => {
            let text = "Meatpuppet pushed Claude's commits".to_owned();
            events::record(conn, item_id, now, &Change::Note { text })
        }
        Ok(Published::Pr { number, url }) => {
            let pr = OpenedPr {
                repo: &ws.repo,
                number,
                url: &url,
            };
            github::adopt(conn, item_id, pr, now)
        }
        Err(e) => {
            let text = format!("Meatpuppet couldn't push Claude's work or open its PR: {e}");
            events::record(conn, item_id, now, &Change::Note { text })?;
            let raised = Urgent {
                id: 0,
                title: format!("Couldn't publish Claude's work on {item_id}"),
                detail: format!("{e}. The commits are in {}.", ws.dir.display()),
                area: items::get(conn, item_id)?.area,
                raised_at: now,
                schedule_id: None,
            };
            urgent::raise(conn, &raised)
        }
    }
}
