use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use crate::config::calendar::Feed;
use crate::config::edit;
use crate::config::settings::Area;
use crate::error::AppResult;

const UPSERT_AREA: &str = "Adds an area of life, or changes one by id: its name, dot colour (CSS hex) and the \
machine (`node`) that runs its schedules. Use list_areas to see machine names.";
const ADD_FEED: &str = "Adds a calendar from its private iCal link (Google: Settings > the calendar > \
\"Secret address in iCal format\"), or replaces one with the same name. Its events become meetings in `area`.";
const CALENDAR_RULES: &str = "Sets which calendar events Meatpuppet skips for every calendar: titles containing \
any of `ignore_titles` (case-insensitive), and optionally all-day events. To hide a single event, use \
update_meeting with hidden instead.";
const MAP_GITHUB: &str = "Files PRs from a GitHub owner (e.g. \"my-org\") or repo (\"me/app\") under an area. \
A repo entry beats its owner's.";
const IGNORE_GITHUB: &str = "Sets the GitHub owners or repos whose PRs Meatpuppet skips entirely.";
const SELF_REVIEW: &str = "Sets the GitHub owners or repos where the user reviews their own PRs, so an unreviewed PR \
shows as waiting on their decision rather than on reviewers. PRs nobody has been asked to review, or has reviewed, are \
treated that way already.";

const INFORMATIONAL: &str = "Sets which checks on an owner's or repo's PRs only inform, e.g. a visual-diff \
report like \"reg\". Their failures don't count as CI failing (so Meatpuppet won't start a fix), and \
their results are noted on the item instead. An empty list clears it.";

pub fn tools() -> Vec<Tool> {
    vec![
        tool("upsert_area", UPSERT_AREA, upsert_area),
        tool("add_calendar", ADD_FEED, add_feed),
        tool(
            "remove_calendar",
            "Stops syncing a calendar, by name.",
            remove_feed,
        ),
        tool("set_calendar_rules", CALENDAR_RULES, calendar_rules),
        tool("map_github_area", MAP_GITHUB, map_github),
        tool("set_github_ignores", IGNORE_GITHUB, github_ignores),
        tool("set_github_self_review", SELF_REVIEW, self_review),
        tool("set_informational_checks", INFORMATIONAL, informational),
    ]
}

fn upsert_area(ctx: &Ctx, area: Area) -> AppResult<()> {
    edit::upsert_area(&ctx.config, &area)
}

fn add_feed(ctx: &Ctx, feed: Feed) -> AppResult<()> {
    let feed = Feed {
        area: ctx.area(&feed.area)?,
        ..feed
    };
    edit::upsert_feed(&ctx.config, &feed)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NameArgs {
    name: String,
}

fn remove_feed(ctx: &Ctx, args: NameArgs) -> AppResult<()> {
    edit::remove_feed(&ctx.config, &args.name)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CalendarRules {
    ignore_titles: Vec<String>,
    /// Skip all-day events (holidays, birthdays). Leave out to keep the current setting.
    ignore_all_day: Option<bool>,
}

fn calendar_rules(ctx: &Ctx, args: CalendarRules) -> AppResult<()> {
    edit::set_calendar_ignores(&ctx.config, &args.ignore_titles, args.ignore_all_day)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MapArgs {
    /// e.g. "my-org" or "me/app"
    owner_or_repo: String,
    /// Area id.
    area: String,
}

fn map_github(ctx: &Ctx, args: MapArgs) -> AppResult<()> {
    edit::map_github(&ctx.config, &args.owner_or_repo, &ctx.area(&args.area)?)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IgnoreArgs {
    ignore: Vec<String>,
}

fn github_ignores(ctx: &Ctx, args: IgnoreArgs) -> AppResult<()> {
    edit::set_github_ignores(&ctx.config, &args.ignore)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SelfReviewArgs {
    owners_or_repos: Vec<String>,
}

fn self_review(ctx: &Ctx, args: SelfReviewArgs) -> AppResult<()> {
    edit::set_github_self_review(&ctx.config, &args.owners_or_repos)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct InformationalArgs {
    /// e.g. "my-org" or "me/app"
    owner_or_repo: String,
    /// Check names as GitHub shows them, e.g. `reg`.
    checks: Vec<String>,
}

fn informational(ctx: &Ctx, args: InformationalArgs) -> AppResult<()> {
    edit::set_informational_checks(&ctx.config, &args.owner_or_repo, &args.checks)
}
