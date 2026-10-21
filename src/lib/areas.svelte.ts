import { api } from "./api";
import type { Area } from "./types";

const STORAGE_KEY = "quarterdeck.area";
export const ALL = "all";

function remembered(): string {
  try {
    return localStorage.getItem(STORAGE_KEY) ?? ALL;
  } catch {
    return ALL;
  }
}

/** The configured areas and which one the user is looking at. */
export const areas = $state({ list: [] as Area[], current: remembered() });

export async function loadAreas(): Promise<void> {
  areas.list = await api.areas();
  if (areas.current !== ALL && !areas.list.some((a) => a.id === areas.current)) areas.current = ALL;
}

export function selectArea(id: string): void {
  areas.current = id;
  try {
    localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Remembering the choice is a convenience; ignore storage failures.
  }
}

/** True when something belongs to the area being viewed. */
export const inArea = (thing: { area: string }): boolean =>
  areas.current === ALL || thing.area === areas.current;

export const findArea = (id: string): Area | undefined => areas.list.find((a) => a.id === id);
