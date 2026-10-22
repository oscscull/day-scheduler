//! Finding command-line tools. Apps opened from Finder don't get the shell's PATH, so look where
//! tools like `claude` and `gh` usually live, then ask the user's login shell, which knows about
//! version managers like nvm.

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// A login profile can hang (a keychain prompt, a slow version manager); give up after this.
const SHELL_LIMIT: Duration = Duration::from_secs(5);
/// How long a "not found" answer stands before the shell is asked again.
const MISS_LIMIT: Duration = Duration::from_mins(10);

/// What the login shell said about one command.
enum Lookup {
    Found(PathBuf),
    Missing,
    /// It didn't answer in time, or couldn't start. Not remembered: it says nothing about the tool.
    NoAnswer,
}

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

fn login_shell(command: &str) -> Lookup {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_owned());
    let Ok(mut child) = Command::new(shell)
        .args(["-lc", &format!("command -v '{command}'")])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Lookup::NoAnswer;
    };
    let started = Instant::now();
    while matches!(child.try_wait(), Ok(None)) {
        if started.elapsed() > SHELL_LIMIT {
            drop(child.kill());
            return Lookup::NoAnswer;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let Ok(out) = child.wait_with_output() else {
        return Lookup::NoAnswer;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let path = PathBuf::from(text.lines().last().unwrap_or_default().trim());
    if path.is_absolute() && path.exists() {
        Lookup::Found(path)
    } else {
        Lookup::Missing
    }
}

fn remembered(command: &str) -> Option<Answer> {
    let found = FOUND.get_or_init(Mutex::default).lock().ok()?;
    found.get(command).filter(|a| a.holds()).cloned()
}

fn remember(command: &str, path: Option<&PathBuf>) {
    if let Ok(mut found) = FOUND.get_or_init(Mutex::default).lock() {
        let answer = Answer {
            path: path.cloned(),
            at: Instant::now(),
        };
        found.insert(command.to_owned(), answer);
    }
}

fn ask_shell(command: &str) -> Option<PathBuf> {
    match login_shell(command) {
        Lookup::Found(path) => {
            remember(command, Some(&path));
            Some(path)
        }
        Lookup::Missing => {
            remember(command, None);
            None
        }
        Lookup::NoAnswer => None,
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
    remembered(command)
        .map_or_else(|| ask_shell(command), |a| a.path)
        .unwrap_or(given)
}

/// PATH with `program`'s own folder first, so a tool installed by a version manager finds its
/// runtime (an npm-installed script finds `node` beside it).
pub fn path_beside(program: &Path) -> Option<OsString> {
    let dir = program.parent().filter(|d| d.is_absolute())?;
    let current = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(dir.to_path_buf()).chain(std::env::split_paths(&current));
    std::env::join_paths(paths).ok()
}
