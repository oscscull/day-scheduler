use chrono::Utc;
use tauri::State;

use crate::app::App;
use crate::claude;
use crate::config::prompts::{Prompt, render};
use crate::domain::activity::Activity;
use crate::error::AppResult;
use crate::store::{activity, urgent};

#[tauri::command]
pub fn dismiss_urgent(app: State<'_, App>, id: i64) -> AppResult<()> {
    app.db.with(|conn| urgent::dismiss(conn, id))
}

#[tauri::command]
pub fn dismiss_all_urgent(app: State<'_, App>) -> AppResult<()> {
    app.db.with(urgent::dismiss_all)
}

/// Hands an urgent item to Claude with the user's instructions. Claude's one-line reply becomes an
/// activity note (which expires like any other) and the item is cleared.
#[tauri::command]
pub async fn resolve_urgent(app: State<'_, App>, id: i64, message: String) -> AppResult<String> {
    let settings = app.settings()?;
    let item = app.db.with(|conn| urgent::find(conn, id))?;
    let vars = [
        ("title", item.title.as_str()),
        ("detail", item.detail.as_str()),
        ("message", message.as_str()),
    ];
    let prompt = render(&app.config, Prompt::ResolveUrgent, &vars)?;
    let reply = claude::ask(&settings.claude_command, &prompt).await?;
    app.db.with(|conn| {
        let note = Activity {
            id: 0,
            text: reply.clone(),
            area: item.area.clone(),
            at: Utc::now(),
            unread: true,
        };
        activity::insert(conn, &note)?;
        urgent::dismiss(conn, id)
    })?;
    Ok(reply)
}
