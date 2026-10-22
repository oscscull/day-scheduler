import { api, errorText } from "./api";
import { fromPeers, tag } from "./peers.svelte";
import type { WorkItem } from "./types";

/** Work items in priority order from every machine, shared by Priorities, Board and In flight. */
export function workItems() {
  const state = $state({ items: [] as WorkItem[], error: "", now: new Date() });

  async function load(): Promise<void> {
    try {
      const [local, remote] = await Promise.all([
        api.workItems(),
        fromPeers<WorkItem[]>("list_items", { include_done: true }),
      ]);
      state.items = [...local, ...remote.flatMap((r) => tag(r.value, r.node))];
      state.now = new Date();
      state.error = "";
    } catch (e) {
      state.error = errorText(e);
    }
  }

  return { state, load };
}

/** Parked items from every machine, for the board's parked list. */
export function parkedItems() {
  const state = $state({ items: [] as WorkItem[], error: "" });

  async function load(): Promise<void> {
    try {
      const [local, remote] = await Promise.all([
        api.parkedItems(),
        fromPeers<WorkItem[]>("list_items", { parked: true, include_done: true }),
      ]);
      state.items = [...local, ...remote.flatMap((r) => tag(r.value, r.node))];
      state.error = "";
    } catch (e) {
      state.error = errorText(e);
    }
  }

  return { state, load };
}
