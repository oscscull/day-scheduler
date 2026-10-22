//! Copies one data folder's database into another, for seeding a dev build from the installed app.

use std::fs;
use std::path::Path;

use based_runtime::RusqliteBackend;

use super::db::{FILE, db_error};
use super::legacy::OLD_FILE;
use crate::error::{AppError, AppResult};

/// Every database file a data folder may hold, including SQLite's sidecars and an imported old one.
fn database_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let imported = format!("{OLD_FILE}.imported");
    [FILE, OLD_FILE, imported.as_str()]
        .iter()
        .flat_map(|n| ["", "-wal", "-shm"].map(|s| dir.join(format!("{n}{s}"))))
        .collect()
}

fn utf8(path: &Path) -> AppResult<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| AppError::Db("a data folder's path isn't UTF-8".to_owned()))
}

/// Replaces `to`'s database with a consistent snapshot of `from`'s (`VACUUM INTO`), safe while the
/// app owning `from` is running. A pre-Based database is copied as is; the next start imports it.
pub async fn snapshot(from: &Path, to: &Path) -> AppResult<()> {
    let name = [FILE, OLD_FILE]
        .into_iter()
        .find(|n| from.join(n).exists())
        .ok_or_else(|| AppError::NotFound(format!("a database in {}", from.display())))?;
    fs::create_dir_all(to)?;
    for file in database_files(to) {
        if file.exists() {
            fs::remove_file(file)?;
        }
    }
    let backend = RusqliteBackend::open(&utf8(&from.join(name))?).map_err(db_error)?;
    let target = utf8(&to.join(name))?.replace('\'', "''");
    backend
        .execute_batch(&format!("VACUUM INTO '{target}'"))
        .await
        .map_err(db_error)
}
