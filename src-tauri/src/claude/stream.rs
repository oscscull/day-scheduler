//! Reads one line of `claude -p --output-format stream-json --verbose`. Only a few fields are used,
//! and anything unknown is ignored, so new event types in Claude Code don't break a run.

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::step::{Step, StepKind};

/// Longest step text kept: one line on the Agent tab.
const STEP_CHARS: usize = 160;

/// Tool inputs worth showing, best first. Paths are cut to the file name.
const DETAIL_KEYS: [&str; 8] = [
    "description",
    "file_path",
    "notebook_path",
    "pattern",
    "query",
    "url",
    "command",
    "prompt",
];

/// The run's final reply.
#[derive(Debug, PartialEq, Eq)]
pub struct Reply {
    pub text: String,
    pub failed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Step(Step),
    Done(Reply),
}

/// The first non-empty line of `text`, cut to [`STEP_CHARS`].
fn clip(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .chars()
        .take(STEP_CHARS)
        .collect()
}

/// `mcp__quarterdeck__post_update` reads better as `post_update`.
fn tool_name(name: &str) -> String {
    name.strip_prefix("mcp__")
        .and_then(|rest| rest.split_once("__"))
        .map_or(name, |(_, tool)| tool)
        .to_owned()
}

fn tool_detail(input: &Value) -> String {
    let Some((key, value)) = DETAIL_KEYS
        .iter()
        .find_map(|k| input.get(k).and_then(Value::as_str).map(|v| (*k, v)))
    else {
        return String::new();
    };
    let shown = if key.ends_with("_path") {
        value.rsplit(['/', '\\']).next().unwrap_or(value)
    } else {
        value
    };
    clip(shown)
}

const fn step(at: DateTime<Utc>, kind: StepKind, tool: Option<String>, text: String) -> Event {
    Event::Step(Step {
        at,
        kind,
        tool,
        text,
    })
}

/// One content block of an assistant or user message.
fn from_block(block: &Value, at: DateTime<Utc>) -> Option<Event> {
    let text_of = |key: &str| block.get(key).and_then(Value::as_str).map(clip);
    match block.get("type").and_then(Value::as_str)? {
        "text" => text_of("text")
            .filter(|t| !t.is_empty())
            .map(|t| step(at, StepKind::Said, None, t)),
        "tool_use" => {
            let name = block.get("name").and_then(Value::as_str)?;
            let detail = block.get("input").map(tool_detail).unwrap_or_default();
            Some(step(at, StepKind::Tool, Some(tool_name(name)), detail))
        }
        "tool_result" if block.get("is_error").and_then(Value::as_bool) == Some(true) => {
            Some(step(
                at,
                StepKind::Failed,
                None,
                text_of("content").unwrap_or_default(),
            ))
        }
        _ => None,
    }
}

fn from_message(event: &Value, at: DateTime<Utc>) -> Vec<Event> {
    event
        .pointer("/message/content")
        .and_then(Value::as_array)
        .map(|blocks| blocks.iter().filter_map(|b| from_block(b, at)).collect())
        .unwrap_or_default()
}

fn summary(event: &Value, at: DateTime<Utc>) -> Vec<Event> {
    let detail = event.get("detail").and_then(Value::as_str).map(clip);
    detail
        .filter(|d| !d.is_empty())
        .map(|d| step(at, StepKind::Doing, None, d))
        .into_iter()
        .collect()
}

fn done(event: &Value) -> Vec<Event> {
    let text = event
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let failed = event.get("is_error").and_then(Value::as_bool) == Some(true);
    vec![Event::Done(Reply {
        text: text.trim().to_owned(),
        failed,
    })]
}

/// What one line of output means. Lines that aren't JSON, or say nothing useful, give nothing.
pub fn parse(line: &str, at: DateTime<Utc>) -> Vec<Event> {
    let Ok(event) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    let kind = event.get("type").and_then(Value::as_str);
    let subtype = event.get("subtype").and_then(Value::as_str);
    match (kind, subtype) {
        (Some("assistant" | "user"), _) => from_message(&event, at),
        (Some("system"), Some("task_summary")) => summary(&event, at),
        (Some("result"), _) => done(&event),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> Event {
        let mut events = parse(line, Utc::now());
        assert_eq!(events.len(), 1, "{line}");
        events.remove(0)
    }

    fn step_of(line: &str) -> Step {
        match one(line) {
            Event::Step(s) => s,
            Event::Done(_) => panic!("expected a step"),
        }
    }

    #[test]
    fn tool_calls_show_the_most_useful_input() {
        let bash = step_of(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"ls defaults","description":"List the defaults directory"}}]}}"#,
        );
        assert_eq!(bash.kind, StepKind::Tool);
        assert_eq!(bash.tool.as_deref(), Some("Bash"));
        assert_eq!(bash.text, "List the defaults directory");
        let read = step_of(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read","input":{"file_path":"/a/b/runner.rs"}}]}}"#,
        );
        assert_eq!(read.text, "runner.rs");
        let mcp = step_of(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"mcp__quarterdeck__post_update","input":{}}]}}"#,
        );
        assert_eq!(mcp.tool.as_deref(), Some("post_update"));
    }

    #[test]
    fn summaries_text_and_errors_are_steps() {
        let doing =
            step_of(r#"{"type":"system","subtype":"task_summary","detail":"Listing files"}"#);
        assert_eq!(
            (doing.kind, doing.text.as_str()),
            (StepKind::Doing, "Listing files")
        );
        assert_eq!(
            parse(
                r#"{"type":"system","subtype":"task_summary","detail":null}"#,
                Utc::now()
            ),
            Vec::<Event>::new()
        );
        let said = step_of(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"\nOn it.\nMore."}]}}"#,
        );
        assert_eq!(said.text, "On it.");
        let failed = step_of(
            r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"no such file","is_error":true}]}}"#,
        );
        assert_eq!(failed.kind, StepKind::Failed);
        assert_eq!(
            parse(
                r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"ok","is_error":false}]}}"#,
                Utc::now()
            ),
            Vec::<Event>::new()
        );
    }

    #[test]
    fn the_result_ends_the_run_and_noise_is_ignored() {
        assert_eq!(
            one(r#"{"type":"result","subtype":"success","is_error":false,"result":" Done. "}"#),
            Event::Done(Reply {
                text: "Done.".to_owned(),
                failed: false
            })
        );
        assert_eq!(parse("not json", Utc::now()), Vec::<Event>::new());
        assert_eq!(
            parse(r#"{"type":"rate_limit_event"}"#, Utc::now()),
            Vec::<Event>::new()
        );
    }
}
