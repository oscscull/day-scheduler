use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use super::{in_area, time};
use crate::domain::commitment::Commitment;
use crate::error::AppResult;
use crate::store::commitments;
use crate::today::upcoming;

pub fn tools() -> Vec<Tool> {
    vec![
        tool(
            "list_meetings",
            "Upcoming meetings and appointments, soonest first. Hidden ones are left out.",
            list,
        ),
        tool(
            "add_meeting",
            "Adds a meeting or appointment the user must attend at a fixed time. Meatpuppet alerts them \
             before it starts. Times are RFC 3339 with an offset.",
            add,
        ),
        tool(
            "update_meeting",
            "Changes a meeting: move it to another area, rename it, or `hidden: true` to stop showing \
             it (e.g. a calendar event the user doesn't care about). Calendar re-syncs keep these changes.",
            update,
        ),
        tool(
            "delete_meeting",
            "Deletes a meeting. A calendar event may come back on the next sync; hide it instead.",
            delete,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Only this area, by id.
    area: Option<String>,
}

fn list(ctx: &Ctx, args: ListArgs) -> AppResult<Vec<Commitment>> {
    let mut all = upcoming(ctx.db.with(commitments::list)?, Utc::now());
    all.retain(|c| in_area(args.area.as_ref(), &c.area));
    Ok(all)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddArgs {
    title: String,
    /// Area id, e.g. "life".
    area: String,
    /// RFC 3339, e.g. 2026-10-23T18:15:00+01:00
    starts_at: String,
    /// RFC 3339
    ends_at: String,
    location: Option<String>,
    /// Video call link, for the alert's Join button.
    join_url: Option<String>,
}

fn add(ctx: &Ctx, args: AddArgs) -> AppResult<()> {
    let c = Commitment {
        id: 0,
        title: args.title,
        area: ctx.area(&args.area)?,
        starts_at: time(&args.starts_at)?,
        ends_at: time(&args.ends_at)?,
        location: args.location,
        join_url: args.join_url,
        source: "manual".to_owned(),
        external_id: None,
        hidden: false,
    };
    ctx.db.with(|conn| commitments::insert(conn, &c))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateArgs {
    id: i64,
    title: Option<String>,
    area: Option<String>,
    starts_at: Option<String>,
    ends_at: Option<String>,
    location: Option<String>,
    hidden: Option<bool>,
}

fn update(ctx: &Ctx, args: UpdateArgs) -> AppResult<()> {
    let area = args.area.as_deref().map(|a| ctx.area(a)).transpose()?;
    let starts = args.starts_at.as_deref().map(time).transpose()?;
    let ends = args.ends_at.as_deref().map(time).transpose()?;
    ctx.db.with(|conn| {
        let old = commitments::get(conn, args.id)?;
        let c = Commitment {
            title: args.title.unwrap_or(old.title),
            area: area.unwrap_or(old.area),
            starts_at: starts.unwrap_or(old.starts_at),
            ends_at: ends.unwrap_or(old.ends_at),
            location: args.location.or(old.location),
            hidden: args.hidden.unwrap_or(old.hidden),
            ..old
        };
        commitments::update(conn, &c)
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: i64,
}

fn delete(ctx: &Ctx, args: IdArgs) -> AppResult<()> {
    ctx.db.with(|conn| commitments::delete(conn, args.id))
}
