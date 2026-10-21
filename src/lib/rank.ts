/**
 * Moves `id` to position `to` among the `visible` ids (e.g. one area's items), and returns the new
 * order of `all` ids. Items that are not visible keep their places relative to each other.
 */
export function reorder(all: string[], visible: string[], id: string, to: number): string[] {
  const shown = visible.filter((v) => v !== id);
  const at = Math.max(0, Math.min(shown.length, to));
  const rest = all.filter((x) => x !== id);
  const before = shown[at];
  if (before !== undefined) {
    rest.splice(rest.indexOf(before), 0, id);
    return rest;
  }
  const last = shown[shown.length - 1];
  rest.splice(last === undefined ? rest.length : rest.indexOf(last) + 1, 0, id);
  return rest;
}
