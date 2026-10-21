use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::ctx::Ctx;
use super::super::tool::{NoArgs, Tool, tool};
use crate::config::edit;
use crate::config::sync::Peer;
use crate::error::AppResult;

const INFO: &str = "How this machine shares Quarterdeck with others: its name, whether it serves peers, the \
local port and token another machine needs, and the peers it knows. Give another machine this \
machine's address (`http://<LocalHostName>.local:<port>` on the same network, or a tunnel URL) and \
token so it can add it with add_peer.";
const ENABLE: &str = "Turns serving this machine's Quarterdeck to peers on or off. With `lan` on it also accepts \
machines on the local network (private addresses only), so two Macs on the same Wi-Fi can sync \
with no tunnel: add each as the other's peer at `http://<LocalHostName>.local:<port>`. With `lan` off \
it listens on this machine only, for use behind a tunnel. Every request needs the token. Takes \
effect when Quarterdeck restarts.";
const ADD: &str = "Adds another machine's Quarterdeck, by its machine name (as used by areas' `node`), the URL its \
tunnel serves, and its sync token. Its data then shows here, and edits to it go back to it.";

pub fn tools() -> Vec<Tool> {
    vec![
        tool("get_sync_info", INFO, info),
        tool("set_sync", ENABLE, set_sync),
        tool("add_peer", ADD, add_peer),
        tool(
            "new_sync_token",
            "Replaces this machine's sync token with five fresh random words and returns them. Peers \
             using the old token stop working until they're given the new one (add_peer again).",
            new_token,
        ),
        tool(
            "remove_peer",
            "Forgets another machine, by name.",
            remove_peer,
        ),
    ]
}

#[derive(Debug, Serialize)]
struct Info {
    this_machine: String,
    serving: bool,
    port: u16,
    token: String,
    peers: Vec<String>,
}

fn info(ctx: &Ctx, _: NoArgs) -> AppResult<Info> {
    let s = ctx.settings()?;
    Ok(Info {
        this_machine: s.node,
        serving: s.sync.enabled,
        port: s.sync.port,
        token: s.sync.token,
        peers: s.sync.peers.into_iter().map(|p| p.name).collect(),
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetArgs {
    enabled: bool,
    port: Option<u16>,
    /// Accept machines on the local network too.
    lan: Option<bool>,
}

fn set_sync(ctx: &Ctx, args: SetArgs) -> AppResult<()> {
    let change = edit::SyncChange {
        enabled: args.enabled,
        port: args.port,
        lan: args.lan,
    };
    edit::set_sync(&ctx.config, change)
}

fn add_peer(ctx: &Ctx, peer: Peer) -> AppResult<()> {
    edit::upsert_peer(&ctx.config, &peer)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NameArgs {
    name: String,
}

fn remove_peer(ctx: &Ctx, args: NameArgs) -> AppResult<()> {
    edit::remove_peer(&ctx.config, &args.name)
}

fn new_token(ctx: &Ctx, _: NoArgs) -> AppResult<String> {
    let token = crate::peers::new_token()?;
    edit::set_sync_token(&ctx.config, &token)?;
    Ok(token)
}
