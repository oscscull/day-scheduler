# Bridge — spec

A "ship's bridge" for work run by Claude. It shows what's in flight and since when, runs recurring work on
a schedule, and only interrupts you when a decision is yours.

> Repo name is `day-scheduler`; "Bridge" is the working name for the product.

## Problem

Claude is good at code, one-off scheduling and computer use. It's bad at:

- **Recurring work.** "Respond to review comments on X the way we did yesterday" gets retyped daily.
- **Checking in.** Nothing notices a PR got comments, or a ticket stalled, unless you ask.
- **Holding state.** With agents driving tickets, it's easy to lose track of work and to have too much
  in progress. Tickets stay open far longer than the work takes.

Bridge holds the state and the clock. Claude does the judgement work.

## Principles

1. **Built for one user first.** Personal tool; optimise for the owner's workflow over generality.
2. **Strict code.** Pedantic clippy, no `unwrap`/`expect`/`panic` in app code, strict TypeScript.
3. **Aggressive single responsibility.** Small functions, small structs, one concern per file, one domain per
   crate. If a file needs a section comment, it should be two files.
4. **Behaviour is configured by prompts.** Any decision about *what* to do (what becomes a work item, when a
   routine may act alone, how to summarise history) lives in a plain-text policy file. Claude can be asked
   to edit those files directly. Code decides *how*; policies decide *what*.
5. **Deterministic where possible.** Polling GitHub, timers, diffing state: app code. Claude is woken only
   when something needs judgement.
6. **Silent unless needed.** Work finishes without input and is logged. The "Needs you" queue and
   notifications are the only interruptions.

## Core concepts

| Concept       | What it is |
|---------------|------------|
| **Node**      | One machine running Bridge + its own Claude (e.g. `work-mac`, `home-mac`). |
| **Work item** | Something in flight: title, short state label, state-since, links, condensed history. |
| **Routine**   | A recurring job: schedule + prompt template + scope + autonomy level. |
| **Run**       | One firing of a routine: start/end, outcome, summary, questions raised. |
| **Feed**      | Read-only inputs pulled into "today": calendar events, Slack mentions, review requests. |
| **Policy**    | A markdown prompt file that decides behaviour (see below). |
| **Ask**       | A question a run raises for you. It sits in the "Needs you" queue until you answer it. |

### Work item

- `title`, `state_label` (≤ ~6 words, e.g. "waiting on review from Sam"), `state_since`
- `status`: `active | waiting | needs_you | done`
- `links`: ticket, PRs, Slack threads, docs
- `history`: condensed timeline entries (who/what/when), summarised by a policy
- `source`: how it was created (manual, Jira, PR, Slack), and `node` that owns it

## Configuration by policy

Each node has a config directory (default `~/.config/bridge/`), watched for changes:

```
~/.config/bridge/
  bridge.toml            # node name, peers, poll intervals: mechanical settings only
  policies/
    intake.md            # what becomes a work item (Jira assigned to me? my open PRs? starred Slack?)
    autonomy.md          # when a routine may act alone vs. must raise an Ask
    state-labels.md      # how to phrase state labels
    history.md           # how to condense history
    today.md             # what goes in the morning "today" view
  routines/
    review-comments.md   # one file per routine
    morning-brief.md
```

A routine file is frontmatter + prompt:

```markdown
---
schedule: "every weekday 09:00"
scope: each-item        # each-item | all-items | today
when: item.has_new_pr_comments
autonomy: act           # act | ask-first
---
Address new review comments on {{item.pr}} following the conventions in {{item.history}}.
Reply on each thread. If a comment asks for a product decision, raise an Ask instead.
```

Defaults ship in the repo under `defaults/` and are copied on first run. To change behaviour, you
(or Claude) edit the files; nothing is hidden in code.

## Architecture

```
┌──────────────────────────── Node (one machine) ────────────────────────────┐
│  Tauri UI  ◄──── events ────  Core (Rust)                                   │
│                               ├─ store       SQLite: items, runs, asks      │
│                               ├─ scheduler   timers → routine firings       │
│                               ├─ watchers    gh / calendar polls (no LLM)   │
│                               ├─ runner      spawns `claude -p` per run     │
│                               ├─ mcp         local MCP server for Claude    │
│                               ├─ notify      native notifications           │
│                               └─ sync        HTTP API for peer nodes        │
└────────────────────────────────────────────────────────────────────────────┘
```

### Running Claude

Bridge runs Claude by spawning headless Claude Code (`claude -p "<rendered prompt>"`) on the user's
existing subscription and connectors (Slack, Calendar, Atlassian, `gh`). No direct API use.

### Claude reporting back

Bridge exposes a local MCP server. Any Claude session on that machine (Bridge-launched, desktop chat,
or Remote Control from a phone) can call it:

- `list_items`, `get_item`
- `update_item(id, state_label, status)`
- `log_history(id, entry)`, `attach_link(id, link)`
- `raise_ask(id, question)` → notification + "Needs you" queue
- `complete_run(run_id, summary)`
- `run_routine(name, node?)`, `answer_ask(id, answer)`

That last pair means a phone session via Remote Control can drive Bridge directly through MCP, with no
clicking. Computer use against the UI stays available as a fallback.

### Multi-machine

- Each node is **authoritative for its own items, routines and runs**. No merge conflicts by design.
- Nodes expose a small authenticated HTTP API (`/state`, `/command`) reachable through a tunnel
  (ngrok, Tailscale, or similar; pluggable).
- Every node shows **one tab per connected node**. Remote tabs are replicas of that node's state;
  actions on them (run routine, answer ask, edit item) are sent as commands to the owning node.
- Auth: shared secret per peer pair to start; upgradeable later.

## Views

1. **Board.** Items grouped by status. Each card shows title, state label, time in state, node.
   Aging is visible: stale items stand out.
2. **Item detail.** Links, condensed history, PR comments and CI, related runs, open asks.
3. **Today.** Calendar, the day's routine schedule, new mentions and review requests.
4. **Needs you.** Every open Ask across all nodes, answerable inline.
5. **Routines.** List, next fire time, last outcome, enable/disable, "run now", open the file.

## Phases

1. **Skeleton.** Workspace + crates, store, manual work items, board + detail UI. Single node.
2. **Routines.** Scheduler, runner (`claude -p`), routine files, runs log, notifications.
3. **MCP.** Local MCP server so Claude sessions update items and raise asks.
4. **Watchers.** `gh` polling for PR comments/CI; calendar feed; intake policy creating items.
5. **Multi-node.** Sync API, tunnel, per-node tabs, remote commands.
6. **Polish.** Visual design pass (Claude design), menu-bar mode, launch at login.

## Open questions

- Store format: SQLite (likely) vs. flat files (more Claude-editable, worse for querying).
- How the runner limits blast radius per routine (allowed tools / permission mode per routine file).
- Tunnel choice and how peers discover each other.
