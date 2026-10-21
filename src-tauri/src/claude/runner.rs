//! Runs one schedule through headless Claude and records the outcome.

use chrono::Utc;

use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::domain::run::{Outcome, Run};
use crate::domain::schedule::Schedule;
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::{runs, urgent};

const SUMMARY_CHARS: usize = 200;

/// Claude's first line, kept short: run history is for glancing, not reading.
fn summary_of(reply: &str) -> String {
    reply
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .chars()
        .take(SUMMARY_CHARS)
        .collect()
}

fn started(schedule: &Schedule) -> Run {
    Run {
        id: 0,
        schedule_id: Some(schedule.id.clone()),
        what: schedule.name.clone(),
        target: String::new(),
        node: schedule.node.clone(),
        started_at: Utc::now(),
        finished_at: None,
        outcome: None,
        summary: None,
    }
}

fn failure(schedule: &Schedule, detail: String) -> Urgent {
    Urgent {
        id: 0,
        title: format!("{} failed", schedule.name),
        detail,
        raised_at: Utc::now(),
        schedule_id: Some(schedule.id.clone()),
    }
}

/// Runs `schedule` now. A failure is recorded on the run and raised as urgent.
pub async fn run(app: &App, schedule: &Schedule) -> AppResult<()> {
    let settings = app.settings()?;
    let id = app.db.with(|conn| runs::start(conn, &started(schedule)))?;
    let vars = [
        ("node", schedule.node.as_str()),
        ("name", schedule.name.as_str()),
        ("instructions", schedule.prompt.as_str()),
    ];
    let reply = match render(&app.config, Prompt::RunSchedule, &vars) {
        Ok(prompt) => super::ask(&settings.claude_command, &prompt).await,
        Err(e) => Err(e),
    };
    let (outcome, summary) = match reply {
        Ok(text) => (Outcome::Ok, summary_of(&text)),
        Err(e) => (Outcome::Failed, summary_of(&e.to_string())),
    };
    app.db.with(|conn| {
        runs::finish(conn, id, outcome, &summary)?;
        if outcome == Outcome::Failed {
            urgent::raise(conn, &failure(schedule, summary.clone()))?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_the_first_real_line() {
        assert_eq!(
            summary_of("\n  \nDid the thing.\nMore detail."),
            "Did the thing."
        );
        assert_eq!(summary_of(&"x".repeat(500)).len(), SUMMARY_CHARS);
    }
}
