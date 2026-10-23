use tauri::State;

use crate::app::App;
use crate::claude::{self, draft};
use crate::config::prompts::{Prompt, render};
use crate::config::schedules;
use crate::error::AppResult;
use crate::peers;

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
    claude::ask(settings.claude()?, &prompt).await
}

/// Turns a plain-English request into a new schedule file. Returns its id.
#[tauri::command]
pub async fn draft_schedule(app: State<'_, App>, request: String) -> AppResult<String> {
    let settings = app.settings()?;
    let areas = settings
        .areas
        .iter()
        .map(|a| format!("{} ({})", a.id, a.name))
        .collect::<Vec<_>>()
        .join(", ");
    let events = crate::domain::fired::KINDS
        .iter()
        .map(|k| format!("- {}: {} Fields: {}.", k.name, k.about, k.fields.join(", ")))
        .collect::<Vec<_>>()
        .join("\n");
    let vars = [
        ("areas", areas.as_str()),
        ("events", events.as_str()),
        ("description", request.as_str()),
    ];
    let prompt = render(&app.config, Prompt::DraftSchedule, &vars)?;
    let reply = claude::ask(settings.claude()?, &prompt).await?;
    let mut schedule = draft::parse(&reply, &request)?;
    schedule.area = settings.known_area(&schedule.area);
    if let Some(id) =
        peers::handoff::create_elsewhere_async(settings.clone(), schedule.clone()).await?
    {
        return Ok(id);
    }
    schedule.id = schedules::free_id(&app.config, &schedule.id);
    schedules::save(&app.config, &schedule)?;
    Ok(schedule.id)
}
