use tauri::State;

use crate::app::App;
use crate::claude::{self, draft};
use crate::config::prompts::{Prompt, render};
use crate::config::schedules;
use crate::error::AppResult;

/// Answers a question from the "Ask Claude" bubble. The reply is not stored.
#[tauri::command]
pub async fn ask_claude(
    app: State<'_, App>,
    page: String,
    context: String,
    message: String,
) -> AppResult<String> {
    let settings = app.settings()?;
    let vars = [
        ("page", page.as_str()),
        ("context", context.as_str()),
        ("message", message.as_str()),
    ];
    let prompt = render(&app.config, Prompt::Chat, &vars)?;
    claude::ask(&settings.claude_command, &prompt).await
}

/// Turns a plain-English request into a new schedule file. Returns its id.
#[tauri::command]
pub async fn draft_schedule(app: State<'_, App>, request: String) -> AppResult<String> {
    let settings = app.settings()?;
    let vars = [
        ("node", settings.node.as_str()),
        ("description", request.as_str()),
    ];
    let prompt = render(&app.config, Prompt::DraftSchedule, &vars)?;
    let reply = claude::ask(&settings.claude_command, &prompt).await?;
    let mut schedule = draft::parse(&reply, &settings.node, &request)?;
    schedule.id = schedules::free_id(&app.config, &schedule.id);
    schedules::save(&app.config, &schedule)?;
    Ok(schedule.id)
}
