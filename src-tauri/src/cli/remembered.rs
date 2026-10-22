//! The login shell's answers, kept so it's asked once per tool: a path while it still exists, a
//! miss for a while, and a shell that didn't answer for a shorter while, so a hung profile only
//! costs its deadline now and then.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::login_shell::{self, Lookup};

const MISS_LIMIT: Duration = Duration::from_mins(10);
const NO_ANSWER_LIMIT: Duration = Duration::from_mins(1);

#[derive(Clone)]
struct Answer {
    lookup: Lookup,
    at: Instant,
}

impl Answer {
    fn holds(&self) -> bool {
        match &self.lookup {
            Lookup::Found(path) => path.exists(),
            Lookup::Missing => self.at.elapsed() < MISS_LIMIT,
            Lookup::NoAnswer => self.at.elapsed() < NO_ANSWER_LIMIT,
        }
    }

    fn path(&self) -> Option<PathBuf> {
        match &self.lookup {
            Lookup::Found(path) => Some(path.clone()),
            Lookup::Missing | Lookup::NoAnswer => None,
        }
    }
}

static FOUND: OnceLock<Mutex<HashMap<String, Answer>>> = OnceLock::new();

fn recall(command: &str) -> Option<Answer> {
    let found = FOUND.get_or_init(Mutex::default).lock().ok()?;
    found.get(command).filter(|a| a.holds()).cloned()
}

fn ask(command: &str) -> Option<PathBuf> {
    let answer = Answer {
        lookup: login_shell::lookup(command),
        at: Instant::now(),
    };
    let path = answer.path();
    if let Ok(mut found) = FOUND.get_or_init(Mutex::default).lock() {
        found.insert(command.to_owned(), answer);
    }
    path
}

pub fn lookup(command: &str) -> Option<PathBuf> {
    recall(command).map_or_else(|| ask(command), |a| a.path())
}
