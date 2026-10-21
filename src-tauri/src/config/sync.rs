//! The `[sync]` section of `quarterdeck.toml`: other machines running Quarterdeck.

use serde::{Deserialize, Serialize};

/// Another machine's Quarterdeck, reached through a tunnel the user runs (Tailscale, ngrok, ssh -L).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Peer {
    /// Its machine name, as used by areas' `node`.
    pub name: String,
    /// e.g. `https://home-mac.tail1234.ts.net:4791` or `http://localhost:4792`
    pub url: String,
    /// The `[sync] token` from that machine's quarterdeck.toml.
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SyncSettings {
    /// Serve this machine's Quarterdeck to peers on `port` (localhost only; a tunnel exposes it).
    pub enabled: bool,
    pub port: u16,
    /// Peers must send this. Generated on first start when empty.
    pub token: String,
    pub peers: Vec<Peer>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 4791,
            token: String::new(),
            peers: Vec::new(),
        }
    }
}

impl SyncSettings {
    pub fn peer(&self, name: &str) -> Option<&Peer> {
        self.peers.iter().find(|p| p.name == name)
    }
}
