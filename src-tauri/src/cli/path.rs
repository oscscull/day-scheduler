//! The PATH a found tool runs with.

use std::ffi::OsString;
use std::path::Path;

/// PATH with `program`'s own folder first, so a tool installed by a version manager finds its
/// runtime (an npm-installed script finds `node` beside it).
pub fn beside(program: &Path) -> Option<OsString> {
    let dir = program.parent().filter(|d| d.is_absolute())?;
    let current = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(dir.to_path_buf()).chain(std::env::split_paths(&current));
    std::env::join_paths(paths).ok()
}
