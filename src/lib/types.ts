// Mirrors the Rust types the backend sends. Keep in step with src-tauri/src/domain and views.

export type Stage = "up_next" | "in_progress" | "in_review" | "done";
export type Status = "decision" | "review" | "ci" | "dependency" | "proceeding" | "queued";

export interface WorkItem {
  id: string;
  title: string;
  stateLabel: string;
  stage: Stage;
  status: Status | null;
  holder: string;
  node: string;
  openedAt: string;
  rank: number;
}

export interface Ask {
  id: number;
  itemId: string;
  question: string;
  context: string;
}

export interface Priority {
  ask: Ask;
  itemId: string;
  itemTitle: string;
}

export interface Commitment {
  id: number;
  title: string;
  startsAt: string;
  endsAt: string;
  location: string | null;
}

export interface Activity {
  id: number;
  text: string;
  at: string;
  unread: boolean;
}

export interface Urgent {
  id: number;
  title: string;
  detail: string;
  raisedAt: string;
  scheduleId: string | null;
}

export interface TodayView {
  urgent: Urgent[];
  upcoming: Commitment[];
  priorities: Priority[];
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

export type EventKind = "ci_failed" | "pr_new_comments";
export type Trigger = { type: "cron"; expr: string } | { type: "event"; kind: EventKind };
export type Autonomy = "ask" | "act";
export type Scope = "each_pr" | "each_item" | "one" | "nothing";

export interface Schedule {
  id: string;
  name: string;
  prompt: string;
  trigger: Trigger;
  node: string;
  scope: Scope;
  autonomy: Autonomy;
  enabled: boolean;
  request: string;
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

export interface AgentView {
  live: Run[];
  schedules: ScheduleView[];
  problems: string[];
  /** Set while every schedule is paused. `until: null` means until resumed. */
  pause: { until: string | null } | null;
}
