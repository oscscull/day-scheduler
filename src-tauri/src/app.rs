use chrono::{Duration, Utc};

use crate::config::Config;
use crate::config::settings::{self, Settings};
use crate::error::AppResult;
use crate::store::Db;
use crate::store::expire::{Ttl, expire};

/// Shared state behind every command.
#[derive(Debug)]
pub struct App {
    pub db: Db,
    pub config: Config,
}

impl App {
    /// Current settings, re-read so file edits apply immediately.
    pub fn settings(&self) -> AppResult<Settings> {
        settings::load(&self.config)
    }

    /// Deletes stored output that has outlived its TTL.
    pub fn expire(&self, settings: &Settings) -> AppResult<()> {
        let ttl = Ttl {
            activity: Duration::days(settings.activity_ttl_days),
            runs: Duration::days(settings.run_ttl_days),
        };
        self.db.with(|conn| expire(conn, Utc::now(), ttl))
    }
}
