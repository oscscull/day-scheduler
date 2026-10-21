import { t } from "../../lib/copy";
import type { Explanation, Trigger } from "../../lib/types";

/** Plain-English "when" for any trigger. Event wording comes from the copy file. */
export function whenText(trigger: Trigger, explanation: Explanation | null): string {
  if (trigger.type === "event") return t(`event.${trigger.kind}`);
  return explanation?.when ?? trigger.expr;
}
