//! Default config files, embedded in the binary and written on first run.
//! Existing files are never overwritten, so user and agent edits survive updates.

use std::path::Path;

use crate::error::AppResult;

macro_rules! default_file {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../defaults/config/", $path)),
        )
    };
}

const FILES: [(&str, &str); 12] = [
    default_file!("README.md"),
    default_file!("bridge.toml"),
    default_file!("prompts/chat.md"),
    default_file!("prompts/draft-schedule.md"),
    default_file!("prompts/resolve-urgent.md"),
    default_file!("prompts/run-schedule.md"),
    default_file!("schedules/morning-brief.md"),
    default_file!("schedules/review-replies.md"),
    default_file!("schedules/ci-watch.md"),
    default_file!("schedules/expense-report.md"),
    default_file!("schedules/stale-check.md"),
    default_file!("schedules/end-of-day.md"),
];

pub fn write_missing(root: &Path) -> AppResult<()> {
    for (relative, contents) in FILES {
        let path = root.join(relative);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)?;
    }
    Ok(())
}
