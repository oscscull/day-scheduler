import { refresh } from "./refresh.svelte";

const REFRESH_MS = 30_000;

/**
 * Loads now and then every 30s. Returns a cleanup function for `$effect`. Because it reads
 * `refresh.version`, the calling effect also reloads whenever something is edited.
 */
export function poll(load: () => Promise<void>): () => void {
  void refresh.version;
  void load();
  const timer = setInterval(() => void load(), REFRESH_MS);
  return () => clearInterval(timer);
}
