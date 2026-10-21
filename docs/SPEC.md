# Bridge spec

Reader: anyone (human or agent) changing Bridge. Rewrite this when the design changes. Setup lives
in [setup.md](setup.md).

## What it is

A calm desktop app that holds the state and the clock for work and life done with Claude. Claude
does the judgement work; Bridge shows what's in flight, runs recurring jobs, and interrupts only
when needed. Natural language is the main way in: the MCP server and Bridge's own chat change
anything the UI shows.

## Hierarchy of attention

1. **Fixed commitments** (meetings): the loudest thing. An on-top countdown with a chime opens a
   few minutes before each.
2. **Priorities**: things where the user's attention unblocks work (design forks, questions).
3. **Claude's activity**: quiet, notification-style updates.

Urgent items (something went wrong or can't wait) show as a red pill next to the next meeting.

## Areas and machines

Everything belongs to an area (Work, Projects, Life by default), defined in `bridge.toml` with the
machine (`node`) that runs it. The switcher filters every view. Each machine keeps its own records
and serves them to peers; the UI merges peers' data and sends edits back to the record's machine.
Connectors and schedules only act for areas this machine runs.

## Views

- **Today**: next meeting, Priorities (asks on the highest-ranked work, derived, never hand-ordered),
  To do (things only the user can do), and "From Claude".
- **Priorities**: everything in play, drag to rank (per machine). The ranking drives Today.
- **Board**: items by stage. **In flight**: open items by who they wait on.
- **Agent**: what's running, every schedule (toggle, edit, delete, run now, create by describing
  it), missed or failed runs, Pause all, connector and peer status, and the MCP connect command.
- **Ask Claude** bubble on every view, given that view's data and the Bridge tools.

## Where things live

- **Config folder** `~/.config/bridge/`, editable by people and agents, re-read on each refresh:
  `bridge.toml` (settings, areas, connectors, sync), `prompts/*.md` (Bridge's own Claude jobs),
  `schedules/<id>.md` (TOML front matter + instructions), `inbox/` (agent-raised urgent items,
  ingested and deleted), `pause.toml` (only while paused). Tools edit `bridge.toml` in place,
  keeping comments, and roll back edits that would break it.
- **Database** (SQLite, WAL, versioned migrations): live state only. Activity, finished runs, done
  to-dos, done items with their history, and past meetings all expire.

## Connectors

Populate by default, never fight overrides. Each keeps the last snapshot it saw and only re-applies
the fields it owns when they change upstream; hidden or re-filed records stay that way.

- **GitHub** (`gh` CLI): open PRs become items; new comments and CI failures fire event schedules.
  PRs with no requested reviewers, or in `self_review` repos, wait on the user.
- **Calendar** (private iCal links): meetings, with recurrence, exceptions and join links.
- **Slack**: no connector; agents bring things in through the tools.

## Running Claude

`claude -p` on the user's subscription, with the Bridge MCP tools always allowed. Schedules can use
other tools only when their file lists them in `allowed_tools`. The scheduler ticks every 30s:
cron runs fire on time, runs missed while asleep are recorded and raised as urgent, and Pause all
stops everything. Event triggers cost nothing until they fire.

## Not built yet

- A database layer on baseddsl, once it can adopt existing databases.
- Slack as a native connector.
