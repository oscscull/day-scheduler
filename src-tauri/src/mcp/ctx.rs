use crate::config::Config;
use crate::config::settings::{self, Settings};
use crate::error::AppResult;
use crate::paths;
use crate::store::Db;

/// What every tool works with: the shared database and config folder.
#[derive(Debug)]
pub struct Ctx {
    pub db: Db,
    pub config: Config,
}

impl Ctx {
    pub fn open() -> AppResult<Self> {
        Self::at(&paths::data_dir()?, paths::config_dir()?)
    }

    pub fn at(data: &std::path::Path, config: std::path::PathBuf) -> AppResult<Self> {
        Ok(Self {
            db: Db::open(data)?,
            config: Config::open(config)?,
        })
    }

    pub fn settings(&self) -> AppResult<Settings> {
        settings::load(&self.config)
    }

    /// `area` if configured, otherwise the first area.
    pub fn area(&self, area: &str) -> AppResult<String> {
        Ok(self.settings()?.known_area(area))
    }
}
