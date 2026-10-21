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

- **Today**: next commitment (with an "Urgent · n" pill when anything is urgent), Priorities, and
  "From Claude". Priorities are derived, never ordered by
  hand: the asks on the highest-ranked work, capped by `priority_limit`.
- **Priorities**: every item in play, drag to rank. The ranking drives Today and Claude's order of work.
- **Board**: items by stage (up next, in progress, in review, done).
- **In flight**: open items grouped by who they wait on (you, Claude, others).
- **Agent**: what Claude is running now, and every schedule (toggle, edit, delete, run now, create by
  describing it). The last run's outcome shows when it was missed or failed. Pause all stops everything
  for today, until a date, or until resumed.
- **Ask Claude** bubble on every view, given that view's data as context. Replies are not stored.

**Urgent** items are things that went wrong or can't wait: a failed run raises one, and an agent raises one
by dropping `inbox/<name>.toml` (`title`, `detail`, optional `schedule_id`) in the config folder, which
Bridge ingests and deletes. Each is dismissed in one click, or resolved by a message to Claude whose
one-line reply becomes an activity note. Dismissed items are deleted.

Status chips: Your decision (warm), Review / CI / Dependency (cool), Proceeding (green), Queued (grey).
No chip means it is moving.

## Where things live

- **Config folder** `~/.config/bridge/`: behaviour, editable by the user or an agent, re-read on each
  refresh. `bridge.toml` (mechanical settings, TTLs), `prompts/*.md` (built-in agent jobs),
  `schedules/<id>.md` (TOML front matter + instructions), `inbox/` (agent-raised urgent items),
  `pause.toml` (present only while everything is paused). Defaults ship in `src-tauri/defaults/config/`
  and are written only when missing.
- **Database** (app data dir, SQLite): live state only. Items and their rank, asks, commitments,
  activity, runs (with outcome: ok, failed, missed), urgent items. Activity and finished runs are deleted after their TTL.
- **Connectors** (GitHub, calendar, Slack): app code, called programmatically. Not built yet; the
  database is filled from `src-tauri/defaults/seed.json` sample data until they are.

## Running Claude

`claude -p` on the user's subscription, no API. Used for the chat bubble, for turning a plain-English
request into a schedule file (`prompts/draft-schedule.md`), for resolving urgent items
(`prompts/resolve-urgent.md`), and for "Run now" (`prompts/run-schedule.md`).

Schedule triggers are `cron` (five fields, explained in plain English in the UI) or `event`
(`ci_failed`, `pr_new_comments`). Events are detected by app code, so they cost nothing until they fire.

## Not built yet

- The scheduler that fires runs on time, records missed runs and respects the pause; the event watchers.
- Connectors; items, asks and meetings are sample data.
- Being designed: areas (Work, Personal projects, Life) replacing machines, and life to-dos.
- Meeting alerts, item detail view, multi-machine sync, agent access to Bridge's data (MCP).
