# Bridge config

Read by: you, and agents asked to change what Bridge does. Bridge re-reads this folder on
every refresh, so edits apply immediately.

- `bridge.toml`: mechanical settings, how long stored output lives, and your areas (e.g. Work,
  Projects, Life), each with the machine that runs its schedules.
- `prompts/`: Bridge's built-in agent jobs. `{{name}}` placeholders are filled in by the app.
- `schedules/`: one file per recurring job; the file name is its id. TOML between `+++`
  lines, then the instructions Claude follows. Copy an existing file to start a new one.
  `allowed_tools` (optional) lists Claude Code tools a run may use without asking, e.g.
  `["Bash(gh pr comment:*)"]`; without it a run can only read and report back.
  `trigger.type` is `cron` (with `expr`) or `event` (with `kind` = `ci_failed` or
  `pr_new_comments`). Events are detected by app code and cost no tokens until they fire.
- `inbox/`: an agent raises an urgent item by writing `inbox/<anything>.toml` with `title` and
  `detail` (and optionally `schedule_id`). Bridge picks it up and deletes the file.
- `pause.toml`: present while every schedule is paused; `until` is when it ends (absent means
  until resumed). Delete it to resume.
