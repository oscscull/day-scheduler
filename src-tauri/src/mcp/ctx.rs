use std::sync::Arc;

use crate::config::Config;
use crate::config::settings::{self, Settings};
use crate::error::AppResult;
use crate::paths;
use crate::store::Db;

/// What every tool works with: the shared database and config folder. Cheap to clone, so each
/// tool call owns one.
#[derive(Debug, Clone)]
pub struct Ctx {
    pub db: Arc<Db>,
    pub config: Config,
}

impl Ctx {
    pub async fn open() -> AppResult<Self> {
        Self::at(&paths::data_dir()?, paths::config_dir()?).await
    }

    pub async fn at(data: &std::path::Path, config: std::path::PathBuf) -> AppResult<Self> {
        Ok(Self {
            db: Arc::new(Db::open(data).await?),
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
