use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use super::{date, in_area};
use crate::domain::todo::Todo;
use crate::error::AppResult;
use crate::store::todos;

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_todos",
            "To-dos: things only the user can do, like errands and bookings. Open ones first, soonest due first.",
            list,
        ),
        tool(
            "add_todo",
            "Adds a to-do for the user, e.g. \"Make dinner\" due Friday. Use for things they must do \
             themselves; use work items for work you or others will carry.",
            add,
        ),
        tool(
            "update_todo",
            "Changes a to-do. Only the fields you pass change.",
            update,
        ),
        tool(
            "complete_todo",
            "Ticks a to-do off (or back on with `done: false`).",
            complete,
        ),
        tool("delete_todo", "Deletes a to-do.", delete),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Only this area, by id.
    area: Option<String>,
    /// Include ticked-off ones. Defaults to false.
    #[serde(default)]
    include_done: bool,
}

fn list(ctx: &Ctx, args: ListArgs) -> AppResult<Vec<Todo>> {
    let mut all = ctx.db.with(todos::list)?;
    all.retain(|t| {
        in_area(args.area.as_ref(), &t.area) && (args.include_done || t.done_at.is_none())
    });
    Ok(all)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddArgs {
    title: String,
    /// Area id, e.g. "life".
    area: String,
    /// YYYY-MM-DD
    due: Option<String>,
}

fn add(ctx: &Ctx, args: AddArgs) -> AppResult<()> {
    let todo = Todo {
        id: 0,
        title: args.title,
        area: ctx.area(&args.area)?,
        due: args.due.as_deref().map(date).transpose()?,
        done_at: None,
    };
    ctx.db.with(|c| todos::insert(c, &todo))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateArgs {
    id: i64,
    title: Option<String>,
    area: Option<String>,
    /// YYYY-MM-DD
    due: Option<String>,
    /// Set true to remove the due date.
    #[serde(default)]
    clear_due: bool,
}

fn update(ctx: &Ctx, args: UpdateArgs) -> AppResult<()> {
    let area = args.area.as_deref().map(|a| ctx.area(a)).transpose()?;
    let due = args.due.as_deref().map(date).transpose()?;
    ctx.db.with(|c| {
        let old = todos::get(c, args.id)?;
        let todo = Todo {
            title: args.title.unwrap_or(old.title),
            area: area.unwrap_or(old.area),
            due: if args.clear_due {
                None
            } else {
                due.or(old.due)
            },
            ..old
        };
        todos::update(c, &todo)
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CompleteArgs {
    id: i64,
    /// Defaults to true.
    done: Option<bool>,
}

fn complete(ctx: &Ctx, args: CompleteArgs) -> AppResult<()> {
    let at = args.done.unwrap_or(true).then(Utc::now);
    ctx.db.with(|c| todos::set_done(c, args.id, at))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: i64,
}

fn delete(ctx: &Ctx, args: IdArgs) -> AppResult<()> {
    ctx.db.with(|c| todos::delete(c, args.id))
}
