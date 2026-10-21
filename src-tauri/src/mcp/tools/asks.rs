use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use super::in_area;
use crate::error::AppResult;
use crate::store::{asks, items};
use crate::today::{Priority, priorities};

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_asks",
            "Every open ask (a decision or question that needs the user), ordered by how high its \
             item is ranked. Today shows the first few.",
            list,
        ),
        tool(
            "add_ask",
            "Asks the user something about an item: a design fork, a question you can't answer. \
             It appears in their Priorities if the item ranks high enough. Put the question in one \
             line and the background in `context`.",
            add,
        ),
        tool("resolve_ask", "Removes an ask once it's answered.", resolve),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Only this area, by id.
    area: Option<String>,
}

fn list(ctx: &Ctx, args: ListArgs) -> AppResult<Vec<Priority>> {
    let mut all = ctx
        .db
        .with(|c| Ok(priorities(&items::list(c)?, &asks::list(c)?, usize::MAX)))?;
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
}

fn add(ctx: &Ctx, args: AddArgs) -> AppResult<()> {
    ctx.db.with(|c| {
        items::get(c, &args.item_id)?;
        asks::insert(c, &args.item_id, &args.question, &args.context)
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ResolveArgs {
    ask_id: i64,
}

fn resolve(ctx: &Ctx, args: ResolveArgs) -> AppResult<()> {
    ctx.db.with(|c| asks::resolve(c, args.ask_id))
}
