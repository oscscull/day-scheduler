import { t } from "./copy";
import { age } from "./time";
import type { Priority } from "./types";

/** The line and its context for a priority, worded by kind. */
export function describe(p: Priority, now: Date): { text: string; context: string } {
  switch (p.kind) {
    case "ask":
      return { text: p.ask.question, context: p.ask.context ? `${p.ask.context} · ${p.itemId}` : p.itemId };
    case "decision":
      return { text: p.itemTitle, context: `${p.itemLabel} · ${p.itemId}` };
    case "fix":
      return { text: t("priority.fix", { title: p.itemTitle }), context: t("priority.fixHint", { id: p.itemId }) };
    case "nudge":
      return {
        text: t("priority.nudge", { holder: p.holder || t("priority.reviewers"), title: p.itemTitle }),
        context: t("priority.nudgeHint", { id: p.itemId, age: age(p.since, now) }),
      };
  }
}

/** Stable across refreshes and machines. */
export const keyOf = (p: Priority): string =>
  `${p.node ?? ""}|${p.kind}|${p.kind === "ask" ? p.ask.id : p.itemId}`;
