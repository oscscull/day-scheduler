import type { CopyKey } from "./copy";
import type { Stage, WorkItem } from "./types";

export const STAGES: readonly Stage[] = ["up_next", "in_progress", "in_review", "done"];

export const isInFlight = (item: WorkItem): boolean =>
  item.stage !== "up_next" && item.stage !== "done";

export type Waiting = "you" | "claude" | "others";

/** Who an in-flight item is waiting on. */
export function waitingOn(item: WorkItem): Waiting {
  if (item.status === "decision") return "you";
  if (item.status === "proceeding" || item.status === "ci_failed" || item.status === null) return "claude";
  return "others";
}

export const WAITING: readonly { id: Waiting; label: CopyKey }[] = [
  { id: "you", label: "inflight.you" },
  { id: "claude", label: "inflight.claude" },
  { id: "others", label: "inflight.others" },
];
