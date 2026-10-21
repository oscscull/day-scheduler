import type { ViewId } from "./views";

/** Which tab is showing; views can switch it, e.g. "+3 more" opens Priorities. */
export const nav = $state({ view: "today" as ViewId });
