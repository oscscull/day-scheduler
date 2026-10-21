# Bridge spec

Reader: anyone (human or agent) changing Bridge. Rewrite this when the design changes.

## What it is

A calm desktop app that holds the state and the clock for work done with Claude. Claude does the
judgement work. Bridge shows what is in flight, runs recurring jobs, and interrupts only when needed.

## Hierarchy of attention

1. **Fixed commitments** (meetings): the loudest thing. Must get the user there on time.
2. **Priorities**: things where the user's attention unblocks work (design forks, questions).
3. **Claude's activity**: quiet, notification-style updates.

## Views

- **Today**: next commitment, Priorities, and "From Claude". Priorities are derived, never ordered by
  hand: the asks on the highest-ranked work, capped by `priority_limit`.
- **Priorities**: every item in play, drag to rank. The ranking drives Today and Claude's order of work.
- **Board**: items by stage (up next, in progress, in review, done).
- **In flight**: open items grouped by who they wait on (you, Claude, others).
- **Agent**: what Claude is running now, and every schedule (toggle, edit, delete, create by describing it).
- **Ask Claude** bubble on every view, given that view's data as context. Replies are not stored.

Status chips: Your decision (warm), Review / CI / Dependency (cool), Proceeding (green), Queued (grey).
No chip means it is moving.

## Where things live

- **Config folder** `~/.config/bridge/`: behaviour, editable by the user or an agent, re-read on each
  refresh. `bridge.toml` (mechanical settings, TTLs), `prompts/*.md` (built-in agent jobs),
  `schedules/<id>.md` (TOML front matter + instructions). Defaults ship in `src-tauri/defaults/config/`
  and are written only when missing.
- **Database** (app data dir, SQLite): live state only. Items and their rank, asks, commitments,
  activity, runs. Activity and finished runs are deleted after their TTL.
- **Connectors** (GitHub, calendar, Slack): app code, called programmatically. Not built yet; the
  database is filled from `src-tauri/defaults/seed.json` sample data until they are.

## Running Claude

`claude -p` on the user's subscription, no API. Used for the chat bubble and for turning a
plain-English request into a schedule file (`prompts/draft-schedule.md`).

Schedule triggers are `cron` (five fields, explained in plain English in the UI) or `event`
(`ci_failed`, `pr_new_comments`). Events are detected by app code, so they cost nothing until they fire.

## Not built yet

- The scheduler that fires runs, and the event watchers.
- Connectors; items, asks and meetings are sample data.
- Being designed: urgent items (raised by an agent or by a missed run), missed-run status on schedules,
  and pausing all schedules (e.g. on holiday).
- Meeting alerts, item detail view, multi-machine sync.
