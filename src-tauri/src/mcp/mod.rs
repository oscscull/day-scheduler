//! A Model Context Protocol server over stdio, so any Claude session can read and change Meatpuppet.
//! Started as `<app binary> mcp`. It works on the same database and config folder as the app.

mod ctx;
mod protocol;
mod tool;
mod tools;

pub use ctx::Ctx;
pub use tool::Tool;

use serde_json::Value;

use crate::error::{AppError, AppResult};

/// Every Meatpuppet tool, for the MCP server and the peer server.
pub fn registry() -> Vec<Tool> {
    tools::all()
}

/// Runs one tool by name.
pub fn call(ctx: &Ctx, tools: &[Tool], name: &str, args: Value) -> AppResult<Value> {
    let tool = tools
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| AppError::BadInput(format!("no tool called {name}")))?;
    (tool.call)(ctx, args)
}

use std::io::{BufRead as _, Write as _};
use std::process::ExitCode;

/// Serves MCP requests from stdin until it closes.
pub fn serve() -> ExitCode {
    let Ok(ctx) = Ctx::open() else {
        return ExitCode::FAILURE;
    };
    let registry = tools::all();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            return ExitCode::FAILURE;
        };
        if let Some(reply) = protocol::handle(&ctx, &registry, &line)
            && (writeln!(stdout, "{reply}").is_err() || stdout.flush().is_err())
        {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::tool::Tool;
    use super::*;

    fn session(name: &str) -> (Ctx, Vec<Tool>) {
        let dir =
            std::env::temp_dir().join(format!("quarterdeck-mcp-{name}-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        let ctx = Ctx::at(&dir.join("data"), dir.join("config")).unwrap();
        (ctx, tools::all())
    }

    fn send(ctx: &Ctx, registry: &[Tool], method: &str, params: &Value) -> Value {
        let line =
            json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        serde_json::from_str(&protocol::handle(ctx, registry, &line).unwrap()).unwrap()
    }

    fn call(ctx: &Ctx, registry: &[Tool], name: &str, args: &Value) -> String {
        let reply = send(
            ctx,
            registry,
            "tools/call",
            &json!({ "name": name, "arguments": args }),
        );
        assert_eq!(reply["result"]["isError"], Value::Null, "{reply}");
        reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn a_session_can_list_tools_and_round_trip_an_item() {
        let (ctx, registry) = session("roundtrip");
        let init = send(
            &ctx,
            &registry,
            "initialize",
            &json!({ "protocolVersion": "2025-06-18" }),
        );
        assert_eq!(init["result"]["serverInfo"]["name"], "quarterdeck");
        let listed = send(&ctx, &registry, "tools/list", &json!({}));
        let all = listed["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = all.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(
            names.contains(&"create_item")
                && names.contains(&"add_todo")
                && names.contains(&"pause_all")
        );
        assert!(all.iter().all(|t| t["inputSchema"]["type"] == "object"));
        let id = call(
            &ctx,
            &registry,
            "create_item",
            &json!({ "title": "Fix login", "area": "work", "key": "JD-1" }),
        );
        assert_eq!(id, "\"JD-1\"");
        call(
            &ctx,
            &registry,
            "update_item",
            &json!({ "id": "JD-1", "stage": "in_review", "status": "review" }),
        );
        let item = call(&ctx, &registry, "get_item", &json!({ "id": "JD-1" }));
        assert!(item.contains("in_review") && item.contains("\"kind\": \"stage\""));
    }

    #[test]
    fn bad_input_is_a_readable_tool_error() {
        let (ctx, registry) = session("bad-input");
        let args = json!({ "title": "x", "area": "life", "due": "friday" });
        let reply = send(
            &ctx,
            &registry,
            "tools/call",
            &json!({ "name": "add_todo", "arguments": args }),
        );
        assert_eq!(reply["result"]["isError"], true);
        assert!(
            reply["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("YYYY-MM-DD")
        );
    }
}
