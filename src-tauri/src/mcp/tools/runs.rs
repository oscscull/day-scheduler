use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{NoArgs, Tool, async_tool, tool};
use crate::domain::step::Step;
use crate::error::AppResult;
use crate::runlog;
use crate::views::live::{self, LiveRun};

const LIST_LIVE_RUNS: &str = "What Claude is running on this machine right now, each with the item it works on \
             and its latest step.";
const GET_RUN_LOG: &str = "The steps a live run has taken so far, oldest first: tool calls, what Claude said, \
             errors. Empty once the run has finished.";

pub fn tools() -> Vec<Tool> {
    vec![
        async_tool("list_live_runs", LIST_LIVE_RUNS, list),
        tool("get_run_log", GET_RUN_LOG, log),
    ]
}

async fn list(ctx: Ctx, _: NoArgs) -> AppResult<Vec<LiveRun>> {
    live::build(&ctx.db).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LogArgs {
    /// The run's id, from `list_live_runs`.
    run_id: i64,
}

fn log(_: &Ctx, args: LogArgs) -> AppResult<Vec<Step>> {
    runlog::read(&runlog::dir()?, args.run_id)
}
