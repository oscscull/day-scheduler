//! Finding command-line tools. Apps opened from Finder don't get the shell's PATH, so look where
//! tools like `claude` and `gh` usually live, then ask the user's login shell, which knows about
//! version managers like nvm.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// A login profile can hang (a keychain prompt, a slow version manager); give up after this.
const SHELL_LIMIT: Duration = Duration::from_secs(5);
/// How long a "not found" answer stands before the shell is asked again.
const MISS_LIMIT: Duration = Duration::from_mins(10);

/// What the login shell said about one command, and when.
#[derive(Clone)]
struct Answer {
    path: Option<PathBuf>,
    at: Instant,
}

impl Answer {
    /// A path that still exists, or a miss recent enough to trust.
    fn holds(&self) -> bool {
        self.path
            .as_ref()
            .map_or_else(|| self.at.elapsed() < MISS_LIMIT, |p| p.exists())
    }
}

static FOUND: OnceLock<Mutex<HashMap<String, Answer>>> = OnceLock::new();

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

fn login_shell(command: &str) -> Option<PathBuf> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_owned());
    let mut child = Command::new(shell)
        .args(["-lc", &format!("command -v '{command}'")])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    while child.try_wait().ok()?.is_none() {
        if started.elapsed() > SHELL_LIMIT {
            drop(child.kill());
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let path = PathBuf::from(text.lines().last()?.trim());
    (path.is_absolute() && path.exists()).then_some(path)
}

fn remembered(command: &str) -> Option<Answer> {
    let found = FOUND.get_or_init(Mutex::default).lock().ok()?;
    found.get(command).filter(|a| a.holds()).cloned()
}

fn remember(command: &str, answer: &Answer) {
    if let Ok(mut found) = FOUND.get_or_init(Mutex::default).lock() {
        found.insert(command.to_owned(), answer.clone());
    }
}

pub fn resolve(command: &str) -> PathBuf {
    let given = PathBuf::from(command);
    if given.is_absolute() || command.contains('/') {
        return given;
    }
    if let Some(path) = usual_places(command) {
        return path;
    }
    let answer = remembered(command).unwrap_or_else(|| {
        let asked = Answer {
            path: login_shell(command),
            at: Instant::now(),
        };
        remember(command, &asked);
        asked
    });
    answer.path.unwrap_or(given)
}
