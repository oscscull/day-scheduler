//! Finding command-line tools. Apps opened from Finder don't get the shell's PATH, so look where
//! tools like `claude` and `gh` usually live.

use std::path::PathBuf;

pub fn resolve(command: &str) -> PathBuf {
    let given = PathBuf::from(command);
    if given.is_absolute() || command.contains('/') {
        return given;
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let candidates = [
        home.join(".local/bin").join(command),
        home.join(".claude/local").join(command),
        PathBuf::from("/opt/homebrew/bin").join(command),
        PathBuf::from("/usr/local/bin").join(command),
    ];
    candidates.into_iter().find(|p| p.exists()).unwrap_or(given)
}
