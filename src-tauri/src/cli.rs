//! Finding command-line tools. Apps opened from Finder don't get the shell's PATH, so look where
//! tools like `claude` and `gh` usually live, then ask the user's login shell, which knows about
//! version managers like nvm.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

/// What the login shell said, so it's asked once per tool while the answer stays valid.
static FOUND: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();

fn usual_places(command: &str) -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = [
        home.join(".local/bin").join(command),
        home.join(".claude/local").join(command),
        PathBuf::from("/opt/homebrew/bin").join(command),
        PathBuf::from("/usr/local/bin").join(command),
    ];
    candidates.into_iter().find(|p| p.exists())
}

fn login_shell(command: &str) -> Option<PathBuf> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_owned());
    let out = Command::new(shell)
        .args(["-lc", &format!("command -v '{command}'")])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let path = PathBuf::from(text.lines().last()?.trim());
    (path.is_absolute() && path.exists()).then_some(path)
}

fn remembered(command: &str) -> Option<PathBuf> {
    let found = FOUND.get_or_init(Mutex::default).lock().ok()?;
    found.get(command).filter(|p| p.exists()).cloned()
}

fn remember(command: &str, path: &Path) {
    if let Ok(mut found) = FOUND.get_or_init(Mutex::default).lock() {
        found.insert(command.to_owned(), path.to_path_buf());
    }
}

pub fn resolve(command: &str) -> PathBuf {
    let given = PathBuf::from(command);
    if given.is_absolute() || command.contains('/') {
        return given;
    }
    if let Some(path) = usual_places(command).or_else(|| remembered(command)) {
        return path;
    }
    login_shell(command).map_or(given, |path| {
        remember(command, &path);
        path
    })
}
