import { api, errorText } from "./api";
import type { WorkItem } from "./types";

/** Work items in priority order, shared by Priorities, Board and In flight. */
export function workItems() {
  const state = $state({ items: [] as WorkItem[], error: "", now: new Date() });

  async function load(): Promise<void> {
    try {
      state.items = await api.workItems();
      state.now = new Date();
      state.error = "";
    } catch (e) {
      state.error = errorText(e);
    }
  }

  return { state, load };
}
