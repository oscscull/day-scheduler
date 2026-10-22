# Setting up Quarterdeck

Reader: a person or agent installing Quarterdeck on a machine, or connecting two machines. Rewrite this
when setup changes.

## One machine

1. Needs: Rust, Node, `gh auth login` done, Claude Code (`claude`) logged in.
2. `git clone https://github.com/oscscull/quarterdeck && cd quarterdeck && npm install`
3. Run it with `npm run tauri dev`, or build an app with `npm run tauri build` and open the bundle in
   `src-tauri/target/release/bundle/macos/`.
4. First start creates `~/.config/quarterdeck/` (settings, prompts, schedules, inbox) and the database
   (`quarterdeck.db`) in `~/Library/Application Support/com.oscscull.quarterdeck/`.
5. In `~/.config/quarterdeck/quarterdeck.toml`, set `node` to this machine's name, check the `[[areas]]`
   (each area's `node` is the machine that runs it), map GitHub owners to areas under
   `[github.areas]`, and add calendars' private iCal links under `[[calendar.feeds]]`.
6. Connect Claude Code: run the command shown under "Connect your Claude sessions" on the Agent
   tab. After that, most setup can be done by asking Claude ("add my work calendar: <link>").

## Two machines

Each machine keeps its own data and serves it to the other. Records stay where they were made;
connectors and schedules only run for areas whose `node` is this machine.

1. Give each machine a different `node`, and the same `[[areas]]` list on both.
2. On each machine, turn sharing on: `[sync] enabled = true` (or Claude: `set_sync`), then restart
   Quarterdeck. A token is written to `[sync] token`; `get_sync_info` shows it.
3. Make each machine reachable. On the same network (simplest): `[sync] lan = true` (or Claude:
   `set_sync` with `lan`), and use `http://<LocalHostName>.local:4791`, where `<LocalHostName>` is
   `scutil --get LocalHostName` on that Mac. Only private addresses may connect, and macOS may ask
   once to allow incoming connections. Use the `.local` name, not an IP: if a company VPN claims your
   home network's IPv4 range (common when it's `10.x`), the name still reaches the other Mac over
   IPv6 link-local (`fe80::`), which VPNs leave on the local network. Across networks, expose the port with a tunnel. With Tailscale, enable
   Serve for the tailnet once (the command prints the link), then on each machine run
   `tailscale serve --bg --tcp 4791 tcp://localhost:4791` and use
   `http://<machine>.<tailnet>.ts.net:4791`. (The HTTPS form, `tailscale serve --bg 4791`, also
   works but needs Tailscale's certificate service reachable; it fails on some networks.) Or use
   `ssh -N -L 4792:localhost:4791 other-mac` (then `http://localhost:4792`).
4. On each machine, add the other as a peer: Claude `add_peer` with its `node` name, URL and token,
   or a `[[sync.peers]]` entry in `quarterdeck.toml`.
5. The Agent tab's Connections list shows whether each peer answers.

A machine without a screen can serve its Quarterdeck with `<binary> serve`.

## Checking it

- Lints and tests: `npm run check`, then in `src-tauri/`: `cargo clippy --all-targets -- -D warnings`
  and `cargo test`.
- The scheduler's last error, if any, is in the database's `meta` table as `scheduler.last_error`.
