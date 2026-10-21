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

- **Today**: next meeting, Priorities (derived from ranked work: asks and decisions, then nudges for
  reviews that have waited past `nudge_after_hours`; never things the system handles itself),
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

- **GitHub** (`gh` CLI): open PRs become items; new comments, CI failing and CI passing fire events.
  PRs with no requested reviewers, or in `self_review` repos, wait on the user.
- **Calendar** (private iCal links): meetings, with recurrence, exceptions and join links.
- **Slack**: no connector; agents bring things in through the tools.

## Running Claude

`claude -p` on the user's subscription, with the Bridge MCP tools always allowed. Schedules can use
other tools only when their file lists them in `allowed_tools`. The scheduler ticks every 30s:
cron runs fire on time, runs missed while asleep are recorded and raised as urgent, and Pause all
stops everything. Event triggers cost nothing until they fire.

CI is Bridge's to watch, never an agent's. When CI finishes on an item a Claude session claimed
(`claim_item`), Bridge resumes that session with the result (`prompts/ci-done.md`). A failure
nobody is on, and no `ci_failed` schedule covers, goes to a new claimed session
(`prompts/fix-ci.md`, `[github] fix_ci`).

The watchdog keeps claimed work moving: agents post updates as they go, and only an item Claude is
carrying that stays quiet past `[watchdog] quiet_minutes` gets a check-in (`prompts/check-in.md`),
at most `max_nudges` times before the user is told. Waiting on CI or reviewers isn't silence.

Questions (`add_ask`) are answered, not ticked off: with one of the agent's `options` or in writing,
on Today or the item. Any open item, and every decision on Today, also takes a plain instruction
("Tell Claude what to do", `tell_claude`), delivered the same way (`prompts/message.md`). Once
nothing on an item waits on the user, it stops showing as their decision. The answer goes on the item's history, and the scheduler resumes the session
that asked (or the one holding the item) with it (`prompts/answer.md`). Sessions are stored with
their working directory, since Claude only resumes a session from the folder it ran in.

Work Bridge starts on a PR (fixing CI, acting on an answer) runs in that repo's project when
`[[projects]]` maps it to a checkout (`set_project`): in a git worktree per item under the data
folder, detached at the PR's head so the user's checkout is untouched, with edits allowed and
commands limited to the project's own Claude permissions (`.claude/settings.local.json` is copied
in). Worktrees are removed once their item is done. Repos with no project get a run that can read
CI but not change code.

Default prompt files Bridge wrote and nobody has edited follow newer defaults
(`.bridge-defaults` records what it wrote); edited ones, and `bridge.toml`, are left alone.

## Not built yet

- A database layer on baseddsl, once it can adopt existing databases.
- Slack as a native connector.
