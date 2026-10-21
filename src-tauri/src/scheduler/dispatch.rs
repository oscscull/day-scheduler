//! Starts one Claude run about an item and records it on the Agent tab while it works.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::claude::{self, Ask, Session};
use crate::config::settings::Settings;
use crate::domain::run::{Outcome, Run};
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::store::runs;

/// Whether a run continues a session or starts one Bridge can come back to.
#[derive(Debug, Clone)]
pub enum Thread {
    Start(SessionRef),
    Resume(SessionRef),
}

impl Thread {
    const fn session(&self) -> &SessionRef {
        match self {
            Self::Start(s) | Self::Resume(s) => s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    /// Shown on the Agent tab, e.g. "Fix CI".
    pub what: &'static str,
    pub item_id: String,
    pub prompt: String,
    pub thread: Thread,
    /// Claude Code tools it may use besides the Bridge tools.
    pub tools: Vec<String>,
}

fn start_record(handle: &AppHandle, settings: &Settings, job: &Job) -> AppResult<i64> {
    let record = Run {
        id: 0,
        schedule_id: None,
        what: job.what.to_owned(),
        target: job.item_id.clone(),
        node: settings.node.clone(),
        started_at: Utc::now(),
        finished_at: None,
        outcome: None,
        summary: None,
    };
    handle.state::<App>().db.with(|c| runs::start(c, &record))
}

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect()
}

async fn ask(command: &str, job: &Job) -> String {
    let s = job.thread.session();
    let session = match job.thread {
        Thread::Start(_) => Session::Start(&s.id),
        Thread::Resume(_) => Session::Resume(&s.id),
    };
    let opts = Ask {
        tools: &job.tools,
        session,
        cwd: s.cwd.as_deref(),
    };
    match claude::ask_with(command, &job.prompt, &opts).await {
        Ok(text) => first_line(&text),
        Err(e) => format!("failed: {e}"),
    }
}

pub fn run(handle: &AppHandle, settings: &Settings, job: Job) -> AppResult<()> {
    let id = start_record(handle, settings, &job)?;
    let command = settings.claude_command.clone();
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let summary = ask(&command, &job).await;
        let outcome = if summary.starts_with("failed: ") {
            Outcome::Failed
        } else {
            Outcome::Ok
        };
        // The run record is for display; losing it changes nothing about the item.
        drop(
            handle
                .state::<App>()
                .db
                .with(|c| runs::finish(c, id, outcome, &summary)),
        );
    });
    Ok(())
}
