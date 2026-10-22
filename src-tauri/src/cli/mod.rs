//! Finding command-line tools. Apps opened from Finder don't get the shell's PATH, so look where
//! tools like `claude` and `gh` usually live, then ask the user's login shell, which knows about
//! version managers like nvm.

mod login_shell;
mod path;
mod remembered;

use std::path::PathBuf;

pub use path::beside as path_beside;

fn usual_places(command: &str) -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = [
        home.join(".local/bin").join(command),
        home.join(".claude/local").join(command),
        PathBuf::from("/opt/homebrew/bin").join(command),
        PathBuf::from("/usr/local/bin").join(command),
        PathBuf::from("/usr/bin").join(command),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub fn resolve(command: &str) -> PathBuf {
    let given = PathBuf::from(command);
    if given.is_absolute() || command.contains('/') {
        return given;
    }
    usual_places(command)
        .or_else(|| remembered::lookup(command))
        .unwrap_or(given)
}
