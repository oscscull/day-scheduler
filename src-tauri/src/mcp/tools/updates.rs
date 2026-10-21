use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use crate::domain::activity::Activity;
use crate::error::AppResult;
use crate::store::activity;

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "post_update",
            "Posts a one-line update to the user's quiet \"From Claude\" feed, e.g. \"Submitted the \
             weekly expense report\" or \"Finished JD-11 and opened PR #821\". Updates expire after a \
             week, so post freely but keep them to one line.",
            post,
        ),
        tool(
            "list_updates",
            "Recent \"From Claude\" updates, newest first.",
            list,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PostArgs {
    text: String,
    /// Area id.
    area: String,
}

fn post(ctx: &Ctx, args: PostArgs) -> AppResult<()> {
    let note = Activity {
        id: 0,
        text: args.text,
        area: ctx.area(&args.area)?,
        at: Utc::now(),
        unread: true,
    };
    ctx.db.with(|c| activity::insert(c, &note))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Defaults to 20.
    limit: Option<u32>,
}

fn list(ctx: &Ctx, args: ListArgs) -> AppResult<Vec<Activity>> {
    ctx.db
        .with(|c| activity::recent(c, args.limit.unwrap_or(20)))
}
