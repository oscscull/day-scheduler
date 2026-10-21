//! Turns Claude's JSON reply from `prompts/draft-schedule.md` into a schedule.

use serde::Deserialize;

use crate::config::schedules::slug;
use crate::domain::schedule::{Autonomy, EventKind, Schedule, Scope, Trigger};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
struct Draft {
    name: String,
    cron: Option<String>,
    event: Option<EventKind>,
    prompt: String,
    scope: Scope,
    autonomy: Autonomy,
}

/// The JSON object inside a reply, ignoring any text around it.
fn json_in(reply: &str) -> AppResult<&str> {
    let start = reply.find('{');
    let end = reply.rfind('}');
    start
        .zip(end)
        .and_then(|(s, e)| reply.get(s..=e))
        .ok_or_else(|| AppError::Claude("the reply had no JSON in it".to_owned()))
}

fn trigger(draft: &Draft) -> AppResult<Trigger> {
    match (draft.event, &draft.cron) {
        (Some(kind), _) => Ok(Trigger::Event { kind }),
        (None, Some(expr)) => Ok(Trigger::Cron { expr: expr.clone() }),
        (None, None) => Err(AppError::Claude("the reply had no timing".to_owned())),
    }
}

/// Builds a new, enabled schedule whose id is a slug of its name.
pub fn parse(reply: &str, node: &str, request: &str) -> AppResult<Schedule> {
    let draft: Draft = serde_json::from_str(json_in(reply)?)?;
    Ok(Schedule {
        id: slug(&draft.name),
        trigger: trigger(&draft)?,
        name: draft.name,
        prompt: draft.prompt,
        node: node.to_owned(),
        scope: draft.scope,
        autonomy: draft.autonomy,
        enabled: true,
        request: request.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPLY: &str = r#"Sure: {"name": "CI watch", "cron": null, "event": "ci_failed",
        "prompt": "Re-run flaky jobs.", "scope": "each_pr", "autonomy": "act"} done"#;

    #[test]
    fn parses_event_drafts_inside_chatter() {
        let s = parse(REPLY, "work-mac", "watch CI").unwrap();
        assert_eq!(
            s.trigger,
            Trigger::Event {
                kind: EventKind::CiFailed
            }
        );
        assert_eq!(s.request, "watch CI");
        assert_eq!(s.id, "ci-watch");
    }

    #[test]
    fn rejects_replies_without_json() {
        assert!(parse("no idea", "", "").is_err());
    }
}
