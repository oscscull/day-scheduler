//! After Claude finishes in a worktree, Quarterdeck does the GitHub side: it pushes the commits and,
//! for an issue, opens the PR and makes the item that PR's. Nothing is published while a question
//! on the item is open, since the work is waiting on the user.

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::turns::TurnSettings;
use crate::connectors::github::{self, OpenedPr};
use crate::domain::event::Change;
use crate::domain::turn::Turn;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::notify;
use crate::store::{Db, asks, events, items, urgent};
use crate::turns::{self, Clock, Pass};
use crate::workspace::{self, Published, Workspace};

pub async fn after_run(
    handle: &AppHandle,
    (item_id, ws): (&str, &Workspace),
    limits: &TurnSettings,
) {
    let app = handle.state::<App>();
    let waiting = asks::for_item(&app.db, item_id)
        .await
        .map_or(true, |open| !open.is_empty());
    if waiting {
        return;
    }
    let published = workspace::publish(ws).await;
    let failed = published.as_ref().err().map(ToString::to_string);
    let recorded = record(&app.db, (item_id, ws, limits), published, Utc::now()).await;
    let problem = recorded.err().map(|e| e.to_string()).or(failed);
    if let Some(problem) = problem {
        // The urgent item and history note carry it; the notification is a courtesy.
        let title = format!("Couldn't publish Claude's work on {item_id}");
        drop(notify::send(handle, &title, &problem));
    }
}

async fn record(
    db: &Db,
    (item_id, ws, limits): (&str, &Workspace, &TurnSettings),
    published: AppResult<Published>,
    now: DateTime<Utc>,
) -> AppResult<()> {
    match published {
        Ok(Published::Nothing) => Ok(()),
        Ok(Published::Pushed) => {
            let text = "Quarterdeck pushed Claude's commits".to_owned();
            events::record(db, item_id, now, &Change::Note { text }).await
        }
        Ok(Published::Pr { number, url }) => {
            let pr = OpenedPr {
                repo: &ws.repo,
                number,
                url: &url,
            };
            github::adopt(db, item_id, pr, Clock { limits, now }).await
        }
        Err(e) => {
            let why = format!("Quarterdeck couldn't push the agent's work or open its PR: {e}");
            turns::pass(db, item_id, Pass::to(Turn::You, why), Clock { limits, now }).await?;
            let raised = Urgent {
                id: 0,
                title: format!("Couldn't publish Claude's work on {item_id}"),
                detail: format!("{e}. The commits are in {}.", ws.dir.display()),
                area: items::get(db, item_id).await?.area,
                raised_at: now,
                schedule_id: None,
                run_id: None,
            };
            urgent::raise(db, &raised).await?;
            Ok(())
        }
    }
}
