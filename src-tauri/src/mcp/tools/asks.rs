use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use super::in_area;
use crate::domain::ask::Ask;
use crate::domain::session::SessionRef;
use crate::error::AppResult;
use crate::store::{asks, items};
use chrono::Utc;

use crate::priorities::{Priority, priorities};

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_asks",
            "Everything that needs the user, in order: asks and items waiting on their decision by \
             item rank, then nudges for items waiting on reviewers too long. Today shows the first few.",
            list,
        ),
        tool(
            "add_ask",
            "Asks the user something about an item: a design fork, a question you can't answer. \
             It appears in their Priorities if the item ranks high enough. Put the question in one \
             line and the background in `context`. Offer `options` when the answer is a choice \
             (e.g. [\"Yes\", \"No\"]); they can always write an answer instead. Pass your session \
             id and working directory so Bridge can resume you with the answer; don't wait or poll \
             for it. Answers are also kept on the item's history.",
            add,
        ),
        tool(
            "answer_ask",
            "Answers an ask on the user's behalf, when they told you the answer.",
            answer,
        ),
        tool(
            "tell_claude",
            "Passes the user's instruction about an item to Claude, e.g. \"merge develop in\". Bridge \
             resumes the session working on it, or starts one in the repo's project. Use when the \
             user tells you what should happen to an item that you won't do yourself in this session.",
            tell,
        ),
        tool(
            "dismiss_ask",
            "Removes an ask that no longer matters, without answering it.",
            dismiss,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Only this area, by id.
    area: Option<String>,
}

fn list(ctx: &Ctx, args: ListArgs) -> AppResult<Vec<Priority>> {
    let rules = ctx.settings()?.priority_rules(Utc::now());
    let mut all = ctx
        .db
        .with(|c| Ok(priorities(&items::list(c)?, &asks::list(c)?, rules)))?;
    all.retain(|p| in_area(args.area.as_ref(), &p.area));
    Ok(all)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddArgs {
    item_id: String,
    question: String,
    /// Where it came from or why it matters, e.g. "Design fork on PR #812".
    #[serde(default)]
    context: String,
    /// Choices to answer with, e.g. `["Yes", "No"]`. Leave out for a written answer.
    #[serde(default)]
    options: Vec<String>,
    /// Your Claude Code session, so the answer can resume you.
    session: Option<SessionRef>,
}

fn add(ctx: &Ctx, args: AddArgs) -> AppResult<()> {
    let ask = Ask {
        id: 0,
        item_id: args.item_id,
        question: args.question,
        context: args.context,
        options: args.options,
        asker: args.session,
    };
    ctx.db.with(|c| {
        items::get(c, &ask.item_id)?;
        asks::insert(c, &ask).map(drop)
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AnswerArgs {
    ask_id: i64,
    answer: String,
}

fn answer(ctx: &Ctx, args: AnswerArgs) -> AppResult<()> {
    ctx.db
        .with(|c| crate::answer::give(c, args.ask_id, &args.answer, Utc::now()))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AskIdArgs {
    ask_id: i64,
}

fn dismiss(ctx: &Ctx, args: AskIdArgs) -> AppResult<()> {
    ctx.db.with(|c| asks::remove(c, args.ask_id))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TellArgs {
    item_id: String,
    text: String,
}

fn tell(ctx: &Ctx, args: TellArgs) -> AppResult<()> {
    ctx.db
        .with(|c| crate::answer::tell(c, &args.item_id, &args.text, Utc::now()))
}
