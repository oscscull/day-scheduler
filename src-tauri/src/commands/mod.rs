//! Thin Tauri command handlers. Each one loads, delegates, and returns.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands must take their arguments and State by value"
)]
#![allow(
    clippy::let_underscore_must_use,
    reason = "tauri::command's async expansion discards a value; not our code"
)]

pub mod areas;
pub mod chat;
pub mod items;
pub mod schedules;
pub mod screens;
pub mod todos;
pub mod urgent;
