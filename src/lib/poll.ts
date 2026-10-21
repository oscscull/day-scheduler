const REFRESH_MS = 30_000;

/** Loads now and then every 30s. Returns a cleanup function for `$effect`. */
export function poll(load: () => Promise<void>): () => void {
  void load();
  const timer = setInterval(() => void load(), REFRESH_MS);
  return () => clearInterval(timer);
}
