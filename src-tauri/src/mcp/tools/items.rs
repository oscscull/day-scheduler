use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, async_tool};
use super::in_area;
use crate::domain::event::Change;
use crate::domain::item::{ItemInput, Stage, Status, WorkItem};
use crate::error::AppResult;
use crate::store::{asks, events, items};
use crate::work;

const LIST_ITEMS: &str = "Work items (tickets, PRs, issues, personal projects) in priority order, highest \
             first. Parked items are left out unless `parked` is set.";
const GET_ITEM: &str = "One item with its open asks and its history.";
const CREATE_ITEM: &str = "Adds a work item to the bottom of the priorities. Use the ticket key as `key` when there \
             is one (e.g. JD-104). Returns the item's id.";
const UPDATE_ITEM: &str = "Changes an item. Only the fields you pass change. Keep `state_label` short and current, \
             e.g. \"waiting on review from Sam\". Set `status` to what it's waiting on, or \
             `clear_status` when it's moving again. Only set `decision` alongside an add_ask \
             saying what the user should decide (with options), so they have something to act on.";
const DELETE_ITEM: &str = "Deletes an item with its asks and history.";
const MOVE_ITEM: &str = "Moves an item to a position in the priorities, 1 being the top.";
const PARK_ITEM: &str = "Parks an item the user has set aside (\"don't worry about that one\", \"it's \
             parked\"), taking it off Today, the board and Claude's queue until unparked (`parked: false`). \
             Syncs keep it parked. Prefer this to deleting a GitHub issue's or PR's item. Put the user's reason \
             in `why` when they give one.";
const ADD_NOTE: &str = "Adds a line to an item's history, e.g. \"opened PR #821\". Use for progress worth \
             remembering; the user sees it in the item's history.";

pub fn tools() -> Vec<Tool> {
    vec![
        async_tool("list_items", LIST_ITEMS, list),
        async_tool("get_item", GET_ITEM, get),
        async_tool("create_item", CREATE_ITEM, create),
        async_tool("update_item", UPDATE_ITEM, update),
        async_tool("delete_item", DELETE_ITEM, delete),
        async_tool("move_item", MOVE_ITEM, move_to),
        async_tool("park_item", PARK_ITEM, park),
        async_tool("add_note", ADD_NOTE, note),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListArgs {
    /// Only this area, by id.
    area: Option<String>,
    /// Only this stage.
    stage: Option<Stage>,
    /// Include done items. Defaults to false.
    #[serde(default)]
    include_done: bool,
    /// List the parked items instead. Defaults to false.
    #[serde(default)]
    parked: bool,
}

async fn list(ctx: Ctx, args: ListArgs) -> AppResult<Vec<WorkItem>> {
    let mut all = if args.parked {
        items::parked(&ctx.db).await?
    } else {
        items::list(&ctx.db).await?
    };
    all.retain(|i| {
        in_area(args.area.as_ref(), &i.area)
            && args.stage.is_none_or(|s| s == i.stage)
            && (args.include_done || i.stage != Stage::Done)
    });
    Ok(all)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: String,
}

async fn get(ctx: Ctx, args: IdArgs) -> AppResult<serde_json::Value> {
    Ok(serde_json::json!({
        "item": items::get(&ctx.db, &args.id).await?,
        "parked": items::is_parked(&ctx.db, &args.id).await?,
        "asks": asks::for_item(&ctx.db, &args.id).await?,
        "history": events::for_item(&ctx.db, &args.id).await?,
    }))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateArgs {
    title: String,
    /// Area id, e.g. "work".
    area: String,
    /// Ticket key to use as the id. A short id is made up if left out.
    key: Option<String>,
    stage: Option<Stage>,
    /// What it's waiting on, if anything.
    status: Option<Status>,
    /// Who has it now, e.g. "you", "Claude", "Sam".
    holder: Option<String>,
    /// Where it's at, in a few words.
    state_label: Option<String>,
    /// URL of the ticket, PR or doc.
    link: Option<String>,
}

async fn create(ctx: Ctx, args: CreateArgs) -> AppResult<String> {
    let input = ItemInput {
        id: args.key.unwrap_or_default(),
        title: args.title,
        state_label: args.state_label.unwrap_or_default(),
        stage: args.stage.unwrap_or(Stage::UpNext),
        status: args.status,
        holder: args.holder.unwrap_or_default(),
        area: ctx.area(&args.area)?,
        link: args.link,
    };
    work::save(&ctx.db, input, Utc::now()).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateArgs {
    id: String,
    title: Option<String>,
    area: Option<String>,
    stage: Option<Stage>,
    status: Option<Status>,
    /// Set true to say it's no longer waiting on anything.
    #[serde(default)]
    clear_status: bool,
    holder: Option<String>,
    state_label: Option<String>,
    link: Option<String>,
}

async fn update(ctx: Ctx, args: UpdateArgs) -> AppResult<String> {
    let area = args.area.as_deref().map(|a| ctx.area(a)).transpose()?;
    let old = items::get(&ctx.db, &args.id).await?;
    let input = ItemInput {
        status: if args.clear_status {
            None
        } else {
            args.status.or(old.status)
        },
        id: old.id,
        title: args.title.unwrap_or(old.title),
        state_label: args.state_label.unwrap_or(old.state_label),
        stage: args.stage.unwrap_or(old.stage),
        holder: args.holder.unwrap_or(old.holder),
        area: area.unwrap_or(old.area),
        link: args.link.or(old.link),
    };
    work::save(&ctx.db, input, Utc::now()).await
}

async fn delete(ctx: Ctx, args: IdArgs) -> AppResult<()> {
    items::delete(&ctx.db, &args.id).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MoveArgs {
    id: String,
    /// 1 is the top of the priorities.
    position: usize,
}

async fn move_to(ctx: Ctx, args: MoveArgs) -> AppResult<()> {
    let mut ids: Vec<String> = items::list(&ctx.db)
        .await?
        .into_iter()
        .map(|i| i.id)
        .filter(|i| *i != args.id)
        .collect();
    items::get(&ctx.db, &args.id).await?;
    ids.insert(
        args.position.saturating_sub(1).min(ids.len()),
        args.id.clone(),
    );
    items::reorder(&ctx.db, &ids).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ParkArgs {
    id: String,
    /// False brings it back. Defaults to true.
    #[serde(default = "yes")]
    parked: bool,
    /// The user's reason, noted in its history.
    why: Option<String>,
}

const fn yes() -> bool {
    true
}

async fn park(ctx: Ctx, args: ParkArgs) -> AppResult<()> {
    let choice = crate::park::Parking {
        parked: args.parked,
        why: args.why.as_deref(),
    };
    crate::park::set(&ctx.db, &args.id, choice, Utc::now()).await
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NoteArgs {
    item_id: String,
    text: String,
}

async fn note(ctx: Ctx, args: NoteArgs) -> AppResult<()> {
    items::get(&ctx.db, &args.item_id).await?;
    events::record(
        &ctx.db,
        &args.item_id,
        Utc::now(),
        &Change::Note { text: args.text },
    )
    .await
}
