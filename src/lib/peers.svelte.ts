import { api, errorText } from "./api";
import { areas } from "./areas.svelte";

export interface PeerStatus {
  name: string;
  error: string | null;
}

/** This machine's name and the other machines it can reach. */
export const machine = $state({ name: "", peers: [] as PeerStatus[] });

export async function loadPeers(): Promise<void> {
  try {
    machine.name = await api.machine();
    machine.peers = await api.peers();
  } catch {
    machine.peers = [];
  }
}

const reachable = (): string[] => machine.peers.filter((p) => !p.error).map((p) => p.name);

/** Calls a Bridge tool on every reachable peer; peers that fail are left out. */
export async function fromPeers<T>(tool: string, args: object = {}): Promise<{ node: string; value: T }[]> {
  const nodes = reachable();
  const results = await Promise.allSettled(nodes.map((node) => api.peerCall<T>(node, tool, args)));
  return results.flatMap((r, i) => {
    const node = nodes[i];
    return r.status === "fulfilled" && node ? [{ node, value: r.value }] : [];
  });
}

/** Marks records as living on another machine, so edits go back there. */
export const tag = <T extends object>(list: T[], node: string): (T & { node: string })[] =>
  list.map((x) => ({ ...x, node }));

/** The peer that runs an area, or undefined when it runs here (or the peer isn't reachable). */
export function ownerOf(area: string): string | undefined {
  const node = areas.list.find((a) => a.id === area)?.node;
  return node && node !== machine.name && reachable().includes(node) ? node : undefined;
}

export const peerError = (e: unknown): string => errorText(e);
