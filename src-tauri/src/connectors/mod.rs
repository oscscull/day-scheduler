//! Connectors pull outside data into Meatpuppet with app code, spending no tokens. What they pull is a
//! starting point: users and agents can hide or edit anything, and syncs respect that.

pub mod calendar;
pub mod github;
pub mod status;
