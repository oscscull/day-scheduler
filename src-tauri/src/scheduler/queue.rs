//! Starts the next queued item once the one before it is in review or done.

use chrono::Utc;
use tauri::{AppHandle, Manager as _};

use crate::app::App;
use crate::config::prompts::{Prompt, render};
use crate::config::settings::Settings;
use crate::error::AppResult;
use crate::queue::{self, Next};
use crate::store::items;

pub fn advance(handle: &AppHandle, settings: &Settings) -> AppResult<()> {
    if settings.queue.items.is_empty() {
        return Ok(());
    }
    let app = handle.state::<App>();
    let all = app.db.with(items::list)?;
    let Next::Start(item) = queue::next(&settings.queue.items, &all) else {
        return Ok(());
    };
    // Another machine runs this area's agents.
    if settings.node_for(&item.area) != settings.node {
        return Ok(());
    }
    let message = render(&app.config, Prompt::Queue, &[])?;
    app.db.with(|c| queue::start(c, item, &message, Utc::now()))
}
