use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::app::App;
use crate::error::{AppError, AppResult};
use crate::peers::client;

/// This machine's name, so the UI knows which records are local.
#[tauri::command]
pub fn machine(app: State<'_, App>) -> AppResult<String> {
    Ok(app.settings()?.node)
}

#[derive(Debug, Serialize)]
pub struct PeerStatus {
    name: String,
    error: Option<String>,
}

/// The configured peers and whether each answers.
#[tauri::command]
pub async fn peers(app: State<'_, App>) -> AppResult<Vec<PeerStatus>> {
    let list = app.settings()?.sync.peers;
    tauri::async_runtime::spawn_blocking(move || {
        list.iter()
            .map(|p| PeerStatus {
                name: p.name.clone(),
                error: client::health(p).err().map(|e| e.to_string()),
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::Desktop(e.to_string()))
}

/// Runs a Meatpuppet tool on a peer: how the UI reads and edits another machine's records.
#[tauri::command]
pub async fn peer_call(
    app: State<'_, App>,
    node: String,
    tool: String,
    args: Value,
) -> AppResult<Value> {
    let peer = app
        .settings()?
        .sync
        .peer(&node)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("peer {node}")))?;
    tauri::async_runtime::spawn_blocking(move || client::call(&peer, &tool, &args))
        .await
        .map_err(|e| AppError::Desktop(e.to_string()))?
}
