//! `npm run dev:seed`: replaces a dev build's config and data with a copy of the installed app's,
//! made safe to work on: paused, serving nobody, reaching no peers, running no Claude.

use std::fs;
use std::path::Path;

use crate::config::{Config, edit, pause};
use crate::error::{AppError, AppResult};
use crate::{paths, profile, runlog, store};

fn copy_dir(from: &Path, to: &Path) -> AppResult<()> {
    if to.exists() {
        fs::remove_dir_all(to)?;
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Quit the dev app first: its database is replaced underneath it.
pub async fn seed() -> AppResult<()> {
    if !profile::DEV {
        return Err(AppError::BadInput(
            "only a dev build copies the installed app's data".to_owned(),
        ));
    }
    let config_dir = paths::config_dir()?;
    copy_dir(&paths::installed_config_dir()?, &config_dir)?;
    let config = Config::open(config_dir)?;
    edit::make_dev_safe(&config)?;
    // Paused until resumed, so copied schedules neither run nor pile up as missed.
    pause::pause(&config, None)?;
    store::snapshot(&paths::installed_data_dir()?, &paths::data_dir()?).await?;
    runlog::clear(&runlog::dir()?)
}
