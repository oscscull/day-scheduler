use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{NoArgs, Tool, tool};
use crate::domain::urgent::Urgent;
use crate::error::AppResult;
use crate::store::urgent;

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_urgent",
            "Open urgent items: things that went wrong or can't wait.",
            list,
        ),
        tool(
            "raise_urgent",
            "Flags something that can't wait, shown as a red pill on Today with a notification. Use \
             sparingly: only for things that went wrong or need the user soon. Most things are asks.",
            raise,
        ),
        tool(
            "dismiss_urgent",
            "Clears an urgent item once it's dealt with.",
            dismiss,
        ),
    ]
}

fn list(ctx: &Ctx, _: NoArgs) -> AppResult<Vec<Urgent>> {
    ctx.db.with(urgent::list)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RaiseArgs {
    title: String,
    #[serde(default)]
    detail: String,
    /// Area id.
    area: String,
    /// Set when it's about a schedule, so the user can re-run it in one click.
    schedule_id: Option<String>,
}

fn raise(ctx: &Ctx, args: RaiseArgs) -> AppResult<()> {
    let item = Urgent {
        id: 0,
        title: args.title,
        detail: args.detail,
        area: ctx.area(&args.area)?,
        raised_at: Utc::now(),
        schedule_id: args.schedule_id,
    };
    ctx.db.with(|c| urgent::raise(c, &item))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: i64,
}

fn dismiss(ctx: &Ctx, args: IdArgs) -> AppResult<()> {
    ctx.db.with(|c| urgent::dismiss(c, args.id))
}
