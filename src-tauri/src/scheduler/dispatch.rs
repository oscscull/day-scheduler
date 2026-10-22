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
use crate::workspace::{self, Kind, Workspace};

/// Whether a run continues a session or starts one Quarterdeck can come back to.
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
    /// Claude Code tools it may use besides the Quarterdeck tools.
    pub tools: Vec<String>,
    /// The worktree to make first, for work in a project. Edits are allowed there.
    pub workspace: Option<Workspace>,
}

impl Job {
    pub const fn new(what: &'static str, item_id: String, prompt: String, thread: Thread) -> Self {
        Self {
            what,
            item_id,
            prompt,
            thread,
            tools: Vec::new(),
            workspace: None,
        }
    }

    pub fn with_tools(self, tools: &[String]) -> Self {
        Self {
            tools: tools.to_vec(),
            ..self
        }
    }

    pub fn in_workspace(self, workspace: Option<Workspace>) -> Self {
        Self { workspace, ..self }
    }
}

async fn start_record(handle: &AppHandle, settings: &Settings, job: &Job) -> AppResult<i64> {
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
    runs::start(&handle.state::<App>().db, &record).await
}

fn first_line(text: &str) -> String {
    text.lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect()
}

/// A new session on an issue starts with the issue's text, which Claude may not be able to fetch.
async fn prompt_for(job: &Job) -> AppResult<String> {
    match (&job.workspace, &job.thread) {
        (Some(ws), Thread::Start(_)) if ws.kind == Kind::Issue => {
            let issue = workspace::issue_text(ws).await?;
            Ok(format!("{}\n\n---\n\n{issue}", job.prompt))
        }
        _ => Ok(job.prompt.clone()),
    }
}

/// `mode` applies to work in a project's worktree.
async fn ask(command: &str, job: &Job, mode: &str) -> String {
    if let Some(ws) = &job.workspace
        && let Err(e) = workspace::prepare(ws).await
    {
        return format!("failed: couldn't make the worktree: {e}");
    }
    let prompt = match prompt_for(job).await {
        Ok(p) => p,
        Err(e) => return format!("failed: couldn't read the issue: {e}"),
    };
    let s = job.thread.session();
    let session = match job.thread {
        Thread::Start(_) => Session::Start(&s.id),
        Thread::Resume(_) => Session::Resume(&s.id),
    };
    let opts = Ask {
        tools: &job.tools,
        session,
        cwd: s.cwd.as_deref(),
        permission_mode: job.workspace.as_ref().map(|_| mode),
    };
    match claude::ask_with(command, &prompt, &opts).await {
        Ok(text) => first_line(&text),
        Err(e) => format!("failed: {e}"),
    }
}

pub async fn run(handle: &AppHandle, settings: &Settings, job: Job) -> AppResult<()> {
    let id = start_record(handle, settings, &job).await?;
    let command = settings.claude_command.clone();
    let mode = settings.project_permission_mode.clone();
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let summary = ask(&command, &job, &mode).await;
        let outcome = if summary.starts_with("failed: ") {
            Outcome::Failed
        } else {
            Outcome::Ok
        };
        if outcome == Outcome::Ok
            && let Some(ws) = &job.workspace
        {
            super::handoff::after_run(&handle, &job.item_id, ws).await;
        }
        // The run record is for display; losing it changes nothing about the item.
        drop(runs::finish(&handle.state::<App>().db, id, outcome, &summary).await);
    });
    Ok(())
}
