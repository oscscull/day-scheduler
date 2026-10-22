//! Dev builds (`npm run tauri dev`) are a separate Quarterdeck with their own app id, data and
//! config, so they run beside the installed app and never share its database, scheduler or window.

/// True in debug builds, which is what `tauri dev` makes.
pub const DEV: bool = cfg!(debug_assertions);

/// The installed app's id. Must match `identifier` in `tauri.conf.json`.
pub const INSTALLED_ID: &str = "com.oscscull.quarterdeck";

/// This build's app id: the single-instance lock and data folder go by it.
pub const APP_ID: &str = if DEV {
    "com.oscscull.quarterdeck.dev"
} else {
    INSTALLED_ID
};

/// The installed app's config folder name, under `~/.config`.
pub const INSTALLED_CONFIG: &str = "quarterdeck";

pub const CONFIG: &str = if DEV {
    "quarterdeck-dev"
} else {
    INSTALLED_CONFIG
};

/// What the window, tray and menus call this build.
pub const NAME: &str = if DEV {
    "Quarterdeck Dev"
} else {
    "Quarterdeck"
};
