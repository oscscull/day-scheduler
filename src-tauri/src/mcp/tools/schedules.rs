use chrono::{Local, Utc};
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{NoArgs, Tool, tool};
use super::time;
use crate::config::{pause, schedules};
use crate::cron::explain::{Explanation, explain};
use crate::domain::schedule::{Autonomy, EventKind, Schedule, Scope, Trigger};
use crate::error::{AppError, AppResult};
use crate::store::run_requests;
use crate::views::agent::{self, AgentView};

const LIST_SCHEDULES: &str = "Every recurring job, with when it runs next, its recent runs, what Claude is running now, \
             and whether everything is paused.";
const CREATE_SCHEDULE: &str = "Creates a recurring job Bridge runs through Claude. Give either `cron` (five fields: \
             minute hour day month weekday) or `event`. Prefer an event when one fits: events cost \
             nothing until they happen. Runs can only read unless `allowed_tools` lists Claude Code \
             tools they may use, e.g. [\"Bash(gh pr comment:*)\"]. Returns the schedule's id.";
const UPDATE_SCHEDULE: &str = "Changes a schedule. Only the fields you pass change.";
const SET_SCHEDULE_ENABLED: &str = "Turns one schedule on or off.";
const RUN_SCHEDULE: &str = "Runs a schedule now (within 30 seconds), even if it's paused.";
const PAUSE_ALL: &str = "Pauses every schedule, e.g. while the user is on holiday. Nothing runs and missed runs \
             aren't flagged. `until` is an RFC 3339 time; leave it out to pause until resumed.";
const EXPLAIN_CRON: &str = "Explains a cron expression in plain English with its next run times.";

pub fn tools() -> Vec<Tool> {
    vec![
        tool("list_schedules", LIST_SCHEDULES, list),
        tool("create_schedule", CREATE_SCHEDULE, create),
        tool("update_schedule", UPDATE_SCHEDULE, update),
        tool("delete_schedule", "Deletes a schedule.", delete),
        tool("set_schedule_enabled", SET_SCHEDULE_ENABLED, set_enabled),
        tool("run_schedule", RUN_SCHEDULE, run),
        tool("pause_all", PAUSE_ALL, pause_all),
        tool("resume_all", "Ends a pause.", resume_all),
        tool("explain_cron", EXPLAIN_CRON, explain_cron),
    ]
}

fn list(ctx: &Ctx, _: NoArgs) -> AppResult<AgentView> {
    let loaded = schedules::list(&ctx.config)?;
    let paused = pause::current(&ctx.config, Utc::now())?;
    ctx.db.with(|c| agent::build(c, loaded, paused))
}

fn trigger(cron: Option<String>, event: Option<EventKind>) -> AppResult<Option<Trigger>> {
    match (cron, event) {
        (Some(_), Some(_)) => Err(AppError::BadInput(
            "give cron or event, not both".to_owned(),
        )),
        (Some(expr), None) => {
            explain(&expr, Local::now(), 1)?;
            Ok(Some(Trigger::Cron { expr }))
        }
        (None, Some(kind)) => Ok(Some(Trigger::Event { kind })),
        (None, None) => Ok(None),
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateArgs {
    name: String,
    /// Area id; the area decides which machine runs it.
    area: String,
    /// What Claude should do each time, written to Claude.
    instructions: String,
    /// e.g. "30 8 * * 1-5" for weekdays at 08:30.
    cron: Option<String>,
    event: Option<EventKind>,
    /// What it runs against. Defaults to nothing.
    scope: Option<Scope>,
    /// "ask" (check with the user before changing anything) or "act". Defaults to ask.
    autonomy: Option<Autonomy>,
    #[serde(default)]
    allowed_tools: Vec<String>,
    /// The user's own words, if they asked for it.
    #[serde(default)]
    request: String,
}

fn create(ctx: &Ctx, args: CreateArgs) -> AppResult<String> {
    let trigger = trigger(args.cron, args.event)?
        .ok_or_else(|| AppError::BadInput("give cron or event".to_owned()))?;
    let schedule = Schedule {
        id: schedules::free_id(&ctx.config, &schedules::slug(&args.name)),
        name: args.name,
        prompt: args.instructions,
        trigger,
        area: ctx.area(&args.area)?,
        scope: args.scope.unwrap_or(Scope::Nothing),
        autonomy: args.autonomy.unwrap_or(Autonomy::Ask),
        enabled: true,
        request: args.request,
        allowed_tools: args.allowed_tools,
    };
    schedules::save(&ctx.config, &schedule)?;
    Ok(schedule.id)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateArgs {
    id: String,
    name: Option<String>,
    area: Option<String>,
    instructions: Option<String>,
    cron: Option<String>,
    event: Option<EventKind>,
    scope: Option<Scope>,
    autonomy: Option<Autonomy>,
    allowed_tools: Option<Vec<String>>,
}

fn update(ctx: &Ctx, args: UpdateArgs) -> AppResult<()> {
    let old = schedules::find(&ctx.config, &args.id)?;
    let schedule = Schedule {
        trigger: trigger(args.cron, args.event)?.unwrap_or(old.trigger),
        area: args
            .area
            .as_deref()
            .map(|a| ctx.area(a))
            .transpose()?
            .unwrap_or(old.area),
        name: args.name.unwrap_or(old.name),
        prompt: args.instructions.unwrap_or(old.prompt),
        scope: args.scope.unwrap_or(old.scope),
        autonomy: args.autonomy.unwrap_or(old.autonomy),
        allowed_tools: args.allowed_tools.unwrap_or(old.allowed_tools),
        ..old
    };
    schedules::save(&ctx.config, &schedule)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IdArgs {
    id: String,
}

fn delete(ctx: &Ctx, args: IdArgs) -> AppResult<()> {
    schedules::delete(&ctx.config, &args.id)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EnabledArgs {
    id: String,
    enabled: bool,
}

fn set_enabled(ctx: &Ctx, args: EnabledArgs) -> AppResult<()> {
    let mut schedule = schedules::find(&ctx.config, &args.id)?;
    schedule.enabled = args.enabled;
    schedules::save(&ctx.config, &schedule)
}

fn run(ctx: &Ctx, args: IdArgs) -> AppResult<()> {
    schedules::find(&ctx.config, &args.id)?;
    ctx.db.with(|c| run_requests::add(c, &args.id))
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PauseArgs {
    /// RFC 3339; leave out to pause until resumed.
    until: Option<String>,
}

fn pause_all(ctx: &Ctx, args: PauseArgs) -> AppResult<()> {
    pause::pause(&ctx.config, args.until.as_deref().map(time).transpose()?)
}

fn resume_all(ctx: &Ctx, _: NoArgs) -> AppResult<()> {
    pause::resume(&ctx.config)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ExplainArgs {
    expr: String,
}

fn explain_cron(_: &Ctx, args: ExplainArgs) -> AppResult<Explanation> {
    explain(&args.expr, Local::now(), 3)
}
