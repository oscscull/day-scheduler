//! Starts one Claude run about an item and records it on the Agent tab while it works.

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager as _};

use super::thread::Thread;
use super::{failed, run_turn};
use crate::app::App;
use crate::claude::{self, Ask, Session};
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::domain::run::{Outcome, Run};
use crate::error::AppResult;
use crate::store::runs;
use crate::workspace::{self, Kind, Workspace};

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

async fn start_record(
    handle: &AppHandle,
    settings: &Settings,
    job: &Job,
    started: DateTime<Utc>,
) -> AppResult<i64> {
    let record = Run {
        id: 0,
        schedule_id: None,
        what: job.what.to_owned(),
        target: job.item_id.clone(),
        node: settings.node.clone(),
        started_at: started,
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

/// How to run a job: `mode` applies to work in a project's worktree, `run` is its record.
struct How {
    /// An error when this build may not run Claude; the run records it as failed.
    command: AppResult<String>,
    mode: String,
    run: i64,
}

async fn ask(how: &How, job: &Job) -> String {
    let command = match &how.command {
        Ok(c) => c,
        Err(e) => return format!("failed: {e}"),
    };
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
        permission_mode: job.workspace.as_ref().map(|_| how.mode.as_str()),
        run: Some(how.run),
    };
    match claude::ask_with(command, &prompt, &opts).await {
        Ok(text) => first_line(&text),
        Err(e) => format!("failed: {e}"),
    }
}

/// What came of a run, once it ended.
struct Ended {
    summary: String,
    started: DateTime<Utc>,
    /// False on a build that may not run Claude, whose runs fail by design.
    may_run: bool,
}

async fn after(handle: &AppHandle, job: &Job, ended: &Ended) -> AppResult<()> {
    let app = handle.state::<App>();
    let limits = app.settings()?.turns;
    let Some(detail) = ended.summary.strip_prefix("failed: ") else {
        let run = run_turn::Finished {
            item_id: &job.item_id,
            started: ended.started,
            said: &ended.summary,
        };
        return run_turn::end(&app.db, run, &limits).await;
    };
    if !ended.may_run {
        return Ok(());
    }
    let failure = failed::Failure {
        item_id: &job.item_id,
        what: job.what,
        detail,
    };
    failed::report(handle, &failure, &limits).await
}

pub async fn run(handle: &AppHandle, settings: &Settings, mut job: Job) -> AppResult<()> {
    let app = handle.state::<App>();
    run_turn::take(&app.db, (&job.item_id, job.what), &settings.turns).await?;
    let rule = render(
        &app.config,
        Prompt::HandOver,
        &[("item", job.item_id.as_str())],
    )?;
    job.prompt = format!("{}\n\n{rule}", job.prompt);
    let started = Utc::now();
    let id = start_record(handle, settings, &job, started).await?;
    let how = How {
        command: settings.claude().map(str::to_owned),
        mode: settings.project_permission_mode.clone(),
        run: id,
    };
    let may_run = how.command.is_ok();
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let summary = ask(&how, &job).await;
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
        let ended = Ended {
            summary,
            started,
            may_run,
        };
        // Nothing is left to tell if even this fails; the run record above still shows it.
        drop(after(&handle, &job, &ended).await);
    });
    Ok(())
}
