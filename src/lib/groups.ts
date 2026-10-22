import type { CopyKey } from "./copy";
import type { Stage, Turn, WorkItem } from "./types";

export const STAGES: readonly Stage[] = ["up_next", "in_progress", "in_review", "done"];

export const isInFlight = (item: WorkItem): boolean =>
  item.stage !== "up_next" && item.stage !== "done";

/** In flight, grouped by whose turn it is. The backend keeps the turn; nothing here guesses it. */
export const TURNS: readonly { id: Turn; label: CopyKey }[] = [
  { id: "you", label: "inflight.you" },
  { id: "agent", label: "inflight.agent" },
  { id: "others", label: "inflight.others" },
  { id: "later", label: "inflight.later" },
];
