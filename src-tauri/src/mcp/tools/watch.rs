use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, async_tool};
use crate::domain::session::SessionRef;
use crate::domain::turn::Turn;
use crate::error::AppResult;
use crate::store::{items, watch};
use crate::turns::{self, Clock, Pass};

const CLAIM: &str = "Say you're working on an item: it becomes the agent's turn. Call this when you \
start. Pass your Claude Code session id and working directory, so Quarterdeck can resume your \
session with CI results, answers and prompts to carry on. Post progress as it happens (add_note, \
update_item's state label) and hand the turn on with pass_turn when you stop. Don't poll CI: when \
it finishes on this item, Quarterdeck resumes your session with the result.";

pub fn tools() -> Vec<Tool> {
    vec![async_tool("claim_item", CLAIM, claim)]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ClaimArgs {
    id: String,
    /// Your Claude Code session (id and working directory), so Quarterdeck can resume you.
    session: Option<SessionRef>,
}

async fn claim(ctx: Ctx, args: ClaimArgs) -> AppResult<()> {
    items::get(&ctx.db, &args.id).await?;
    if let Some(session) = &args.session {
        watch::claim(&ctx.db, &args.id, session).await?;
    }
    let limits = ctx.settings()?.turns;
    let pass = Pass::to(Turn::Agent, "a session took it on");
    turns::pass(&ctx.db, &args.id, pass, Clock::now(&limits)).await
}
