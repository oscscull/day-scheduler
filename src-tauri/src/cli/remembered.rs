//! The login shell's answers, kept so it's asked once per tool: a path while it still exists, a
//! miss for a while. A shell that didn't answer isn't remembered.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::login_shell::{self, Lookup};

const MISS_LIMIT: Duration = Duration::from_mins(10);

#[derive(Clone)]
struct Answer {
    path: Option<PathBuf>,
    at: Instant,
}

impl Answer {
    fn holds(&self) -> bool {
        self.path
            .as_ref()
            .map_or_else(|| self.at.elapsed() < MISS_LIMIT, |p| p.exists())
    }
}

static FOUND: OnceLock<Mutex<HashMap<String, Answer>>> = OnceLock::new();

fn recall(command: &str) -> Option<Answer> {
    let found = FOUND.get_or_init(Mutex::default).lock().ok()?;
    found.get(command).filter(|a| a.holds()).cloned()
}

fn keep(command: &str, path: Option<&PathBuf>) {
    if let Ok(mut found) = FOUND.get_or_init(Mutex::default).lock() {
        let answer = Answer {
            path: path.cloned(),
            at: Instant::now(),
        };
        found.insert(command.to_owned(), answer);
    }
}

fn ask(command: &str) -> Option<PathBuf> {
    match login_shell::lookup(command) {
        Lookup::Found(path) => {
            keep(command, Some(&path));
            Some(path)
        }
        Lookup::Missing => {
            keep(command, None);
            None
        }
        Lookup::NoAnswer => None,
    }
}

pub fn lookup(command: &str) -> Option<PathBuf> {
    recall(command).map_or_else(|| ask(command), |a| a.path)
}
