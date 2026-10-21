use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;

use super::super::ctx::Ctx;
use super::super::tool::{Tool, tool};
use crate::config::edit;
use crate::domain::item::{ItemInput, Status};
use crate::error::AppResult;
use crate::store::{items, watch};
use crate::work;

const CLAIM: &str = "Say you're working on an item. Call this when you start. Pass your Claude Code session id if \
you know it, so that if you go quiet Bridge can resume your session and ask for an update rather \
than start a new one. Then post progress as it happens: add_note for steps, update_item for the \
state label, add_ask when you need the user. Don't poll CI: when it finishes on this item, Bridge \
resumes your session with the result. Long silences get a check-in; repeated silence is flagged \
to the user.";
const SETTINGS: &str = "Tunes the watchdog: how long an item Claude holds may go without an update before Bridge \
checks in (quiet_minutes), how many check-ins before it tells the user instead (max_nudges), or \
turns it off. Only the fields you pass change.";

pub fn tools() -> Vec<Tool> {
    vec![
        tool("claim_item", CLAIM, claim),
        tool("set_watchdog", SETTINGS, settings),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ClaimArgs {
    id: String,
    /// Your Claude Code session id, if known.
    session_id: Option<String>,
}

fn claim(ctx: &Ctx, args: ClaimArgs) -> AppResult<()> {
    ctx.db.with(|c| {
        let item = items::get(c, &args.id)?;
        let input = ItemInput {
            status: item.status.or(Some(Status::Proceeding)),
            holder: "Claude".to_owned(),
            id: item.id,
            title: item.title,
            state_label: item.state_label,
            stage: item.stage,
            area: item.area,
            link: item.link,
        };
        work::save(c, input, Utc::now())?;
        watch::claim(c, &args.id, args.session_id.as_deref())
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SettingsArgs {
    enabled: Option<bool>,
    quiet_minutes: Option<i64>,
    max_nudges: Option<u32>,
}

fn settings(ctx: &Ctx, args: SettingsArgs) -> AppResult<()> {
    edit::set_watchdog(
        &ctx.config,
        args.enabled,
        args.quiet_minutes,
        args.max_nudges,
    )
}
