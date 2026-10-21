/** Bumped after any edit so every open view reloads. */
export const refresh = $state({ version: 0 });

export function changed(): void {
  refresh.version += 1;
}
