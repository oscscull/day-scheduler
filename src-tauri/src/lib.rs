/// Demo command: greets `name`. Placeholder until phase 1 replaces it.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

/// Builds and runs the Tauri application.
///
/// # Errors
/// Returns an error if the Tauri runtime fails to start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(
    clippy::exit,
    reason = "generate_context! expands to code calling process::exit; not ours"
)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
}
