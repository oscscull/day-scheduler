import { listen } from "@tauri-apps/api/event";

/** Bumped after any edit so every open view reloads. */
export const refresh = $state({ version: 0 });

export function changed(): void {
  refresh.version += 1;
}

/** Sent by the backend when a connector sync lands (e.g. the one coming back to the window starts). */
const SYNCED = "synced";

/** Reloads when the window comes back into focus, and again when the sync that starts lands.
 * Returns the cleanup. */
export function reloadOnReturn(): () => void {
  window.addEventListener("focus", changed);
  const unlisten = listen(SYNCED, changed);
  return () => {
    window.removeEventListener("focus", changed);
    void unlisten.then((stop) => stop());
  };
}
