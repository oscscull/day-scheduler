import type { Priority } from "./types";

/** The line and its context for a priority, worded by kind. */
export function describe(p: Priority): { text: string; context: string } {
  switch (p.kind) {
    case "ask":
      return { text: p.ask.question, context: p.ask.context ? `${p.ask.context} · ${p.itemId}` : p.itemId };
    case "yours":
      return { text: p.itemTitle, context: `${p.itemLabel} · ${p.itemId}` };
  }
}

/** Stable across refreshes and machines. */
export const keyOf = (p: Priority): string =>
  `${p.node ?? ""}|${p.kind}|${p.kind === "ask" ? p.ask.id : p.itemId}`;
