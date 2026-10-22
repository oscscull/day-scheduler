//! What each live run is doing: one small file per run in the data folder, written by the app as
//! Claude streams and deleted when the run ends. Files rather than the database, so the MCP and peer
//! servers can read them, and nothing outlives its run.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::step::Step;
use crate::error::AppResult;
use crate::paths;

/// Steps kept per run: enough to follow along, never a transcript.
const MAX_STEPS: usize = 60;

pub fn dir() -> AppResult<PathBuf> {
    Ok(paths::data_dir()?.join("live"))
}

fn file(dir: &Path, run: i64) -> PathBuf {
    dir.join(format!("{run}.json"))
}

/// Writes one run's steps as they arrive. Dropping it deletes the file.
#[derive(Debug)]
pub struct Writer {
    path: PathBuf,
    steps: VecDeque<Step>,
}

impl Writer {
    pub fn create(dir: &Path, run: i64) -> AppResult<Self> {
        fs::create_dir_all(dir)?;
        let writer = Self {
            path: file(dir, run),
            steps: VecDeque::new(),
        };
        writer.save()?;
        Ok(writer)
    }

    pub fn push(&mut self, step: Step) -> AppResult<()> {
        if self.steps.len() == MAX_STEPS {
            self.steps.pop_front();
        }
        self.steps.push_back(step);
        self.save()
    }

    /// Writes beside the file, then renames, so a reader never sees half a file.
    fn save(&self) -> AppResult<()> {
        let part = self.path.with_extension("part");
        fs::write(&part, serde_json::to_vec(&self.steps)?)?;
        Ok(fs::rename(part, &self.path)?)
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        // Already gone is fine: the next start clears the folder anyway.
        drop(fs::remove_file(&self.path));
    }
}

/// A run's steps so far, oldest first. Empty once it has finished.
pub fn read(dir: &Path, run: i64) -> AppResult<Vec<Step>> {
    match fs::read(file(dir, run)) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

/// What a run did last, if it has done anything yet.
pub fn latest(dir: &Path, run: i64) -> Option<Step> {
    read(dir, run).ok().and_then(|mut steps| steps.pop())
}

/// Forgets every run's steps; at start-up no run can still be live.
pub fn clear(dir: &Path) -> AppResult<()> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::step::StepKind;

    fn step(text: &str) -> Step {
        Step {
            at: Utc::now(),
            kind: StepKind::Said,
            tool: None,
            text: text.to_owned(),
        }
    }

    #[test]
    fn keeps_the_latest_steps_and_forgets_them_when_done() {
        let dir = std::env::temp_dir().join(format!("quarterdeck-runlog-{}", std::process::id()));
        clear(&dir).unwrap();
        let mut writer = Writer::create(&dir, 7).unwrap();
        assert_eq!(read(&dir, 7).unwrap(), Vec::<Step>::new());
        for n in 0..=MAX_STEPS {
            writer.push(step(&n.to_string())).unwrap();
        }
        let steps = read(&dir, 7).unwrap();
        assert_eq!(steps.len(), MAX_STEPS);
        assert_eq!(steps.first().unwrap().text, "1");
        assert_eq!(latest(&dir, 7).unwrap().text, MAX_STEPS.to_string());
        drop(writer);
        assert_eq!(read(&dir, 7).unwrap(), Vec::<Step>::new());
        assert!(latest(&dir, 8).is_none());
        clear(&dir).unwrap();
    }
}
