// Mirrors the Rust types the backend sends. Keep in step with src-tauri/src/domain and views.

export type Stage = "up_next" | "in_progress" | "in_review" | "done";
export type Status = "decision" | "review" | "ci" | "ci_failed" | "dependency" | "approved" | "proceeding" | "queued";

export interface WorkItem {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: string;
  title: string;
  stateLabel: string;
  stage: Stage;
  status: Status | null;
  holder: string;
  area: string;
  link: string | null;
  /** "manual", or the connector that made it, e.g. "github". */
  source: string;
  externalId: string | null;
  openedAt: string;
  stateSince: string;
  rank: number;
}

/** What the UI sends to create (empty id) or edit an item. */
export interface ItemInput {
  id: string;
  title: string;
  stateLabel: string;
  stage: Stage;
  status: Status | null;
  holder: string;
  area: string;
  link: string | null;
}

export type Change =
  | { kind: "created" }
  | { kind: "stage"; stage: Stage }
  | { kind: "status"; status: Status | null }
  | { kind: "label"; label: string }
  | { kind: "note"; text: string };

export interface ItemEvent {
  id: number;
  itemId: string;
  at: string;
  change: Change;
}

export interface ItemDetail {
  item: WorkItem;
  asks: Ask[];
  history: ItemEvent[];
}

export interface CommitmentInput {
  title: string;
  area: string;
  startsAt: string;
  endsAt: string;
  location: string | null;
}

export interface Ask {
  id: number;
  itemId: string;
  question: string;
  context: string;
  /** Choices to answer with. Empty means a written answer; one can always be written. */
  options: string[];
}

/** Something that needs the user, derived from the work. `kind` says why. */
export type Priority = (
  | { kind: "ask"; ask: Ask }
  | { kind: "decision" }
  | { kind: "nudge" }
) & {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  itemId: string;
  itemTitle: string;
  itemLabel: string;
  holder: string;
  area: string;
  since: string;
};

export interface Commitment {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: number;
  title: string;
  area: string;
  startsAt: string;
  endsAt: string;
  location: string | null;
  joinUrl: string | null;
  source: string;
  externalId: string | null;
  hidden: boolean;
}

export interface Activity {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: number;
  text: string;
  area: string;
  at: string;
  unread: boolean;
}

/** A part of life from quarterdeck.toml, e.g. Work or Life. */
export interface Area {
  id: string;
  name: string;
  color: string;
  node: string;
}

export interface Todo {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: number;
  title: string;
  area: string;
  /** A calendar date, "YYYY-MM-DD". */
  due: string | null;
  doneAt: string | null;
}

export interface Urgent {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: number;
  title: string;
  detail: string;
  area: string;
  raisedAt: string;
  scheduleId: string | null;
}

export interface TodayView {
  urgent: Urgent[];
  upcoming: Commitment[];
  priorities: Priority[];
  priorityLimit: number;
  todos: Todo[];
  activity: Activity[];
  running: number;
  inFlight: number;
}

export type Outcome = "ok" | "failed" | "missed";

export interface Run {
  id: number;
  scheduleId: string | null;
  what: string;
  target: string;
  node: string;
  startedAt: string;
  finishedAt: string | null;
  outcome: Outcome | null;
  summary: string | null;
}

export type EventKind = "ci_failed" | "ci_passed" | "pr_new_comments";
export type Trigger = { type: "cron"; expr: string } | { type: "event"; kind: EventKind };
export type Autonomy = "ask" | "act";
export type Scope = "each_pr" | "each_item" | "one" | "nothing";

export interface Schedule {
  /** Set when it lives on another machine; edits go back there. */
  node?: string;
  id: string;
  name: string;
  prompt: string;
  trigger: Trigger;
  area: string;
  scope: Scope;
  autonomy: Autonomy;
  enabled: boolean;
  request: string;
  allowedTools: string[];
}

export type Unit = "minute" | "hour" | "day" | "month" | "weekday";

export interface CronField {
  unit: Unit;
  value: string;
  means: string;
}

export interface Explanation {
  when: string;
  fields: CronField[];
  nextRuns: string[];
}

export interface ScheduleView extends Schedule {
  explanation: Explanation | null;
  runs: Run[];
}

export type StepKind = "doing" | "tool" | "said" | "failed";

/** One step of a live run. `tool` is set for tool calls. */
export interface Step {
  at: string;
  kind: StepKind;
  tool: string | null;
  text: string;
}

/** A run going now, with the item it works on and its latest step. */
export interface LiveRun extends Run {
  title: string | null;
  now: Step | null;
}

export interface AgentView {
  live: LiveRun[];
  schedules: ScheduleView[];
  problems: string[];
  /** Set while every schedule is paused. `until: null` means until resumed. */
  pause: { until: string | null } | null;
  connections: Connection[];
}

export interface Connection {
  name: string;
  lastSync: string | null;
  error: string | null;
}

/** Whether the user's own Claude Code sessions can reach Quarterdeck. */
export type ClaudeConnection =
  | { state: "connected" }
  | { state: "missing"; command: string }
  | { state: "elsewhere"; command: string };

/** A synced calendar. Its private link stays in quarterdeck.toml. */
export interface Calendar {
  name: string;
  area: string;
}

/** A to-do as the edit form saves it. `due: null` clears the date. */
export interface TodoEdit {
  id: number;
  title: string;
  area: string;
  due: string | null;
}

/** What the user does about something that needs them. */
export type Response =
  | { kind: "answer"; askId: number; text: string }
  | { kind: "dismiss"; askId: number }
  | { kind: "tell"; itemId: string; text: string };

/** What came of a reply: "gone" when the question was already answered or dismissed. */
export type ReplyOutcome = "done" | "gone";
