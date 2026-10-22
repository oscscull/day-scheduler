use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, async_tool, tool};
use super::time;
use crate::config::edit;
use crate::config::turns::TurnChange;
use crate::domain::turn::Turn;
use crate::error::{AppError, AppResult};
use crate::store::items;
use crate::turns::{self, Clock, Pass};

const PASS: &str = "Hands an item's turn on, saying what you did. Every item in flight is someone's \
turn with a clock on it, so nothing waits forever. `to`: `agent` to carry on yourself (Quarterdeck \
prompts you again; the clock doesn't restart), `others` when it waits on reviewers, CI or another \
team (say who in `waiting_on`; it comes back after a while, to `then` if given), `you` when the \
user must decide or act, `later` to put it off until `until`. `did` is required: one line on what \
you did or why you stopped.";
const LIMITS: &str = "Sets how long each turn lasts before Quarterdeck acts: agent_hours (then back \
to the user), you_remind_hours (between reminders), others_hours (then to others_then), \
later_hours, others_then (\"you\" or \"agent\"), and quiet_minutes (how long an agent session \
outside Quarterdeck may go quiet before it's prompted). Only the fields you pass change.";

pub fn tools() -> Vec<Tool> {
    vec![
        async_tool("pass_turn", PASS, pass),
        tool("set_turn_limits", LIMITS, limits),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PassArgs {
    id: String,
    to: Turn,
    /// What you did, or why you stopped, in one line.
    did: String,
    /// For `others`: who it waits on, e.g. "CI" or reviewers' names.
    waiting_on: Option<String>,
    /// When the turn runs out, as RFC 3339. Defaults to the configured length.
    until: Option<String>,
    /// For `others` or `later`: who gets the turn after.
    then: Option<Turn>,
}

async fn pass(ctx: Ctx, args: PassArgs) -> AppResult<()> {
    if args.did.trim().is_empty() {
        return Err(AppError::BadInput("say what you did in `did`".to_owned()));
    }
    items::get(&ctx.db, &args.id).await?;
    let pass = Pass {
        to: args.to,
        why: args.did,
        waiting_on: args.waiting_on,
        until: args.until.as_deref().map(time).transpose()?,
        then: args.then,
    };
    let limits = ctx.settings()?.turns;
    turns::pass(&ctx.db, &args.id, pass, Clock::now(&limits)).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LimitArgs {
    agent_hours: Option<i64>,
    you_remind_hours: Option<i64>,
    others_hours: Option<i64>,
    later_hours: Option<i64>,
    others_then: Option<Turn>,
    quiet_minutes: Option<i64>,
}

fn limits(ctx: &Ctx, args: LimitArgs) -> AppResult<()> {
    let change = TurnChange {
        agent_hours: args.agent_hours,
        you_remind_hours: args.you_remind_hours,
        others_hours: args.others_hours,
        later_hours: args.later_hours,
        others_then: args.others_then,
        quiet_minutes: args.quiet_minutes,
    };
    edit::set_turns(&ctx.config, change)
}
