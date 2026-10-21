import en from "../copy/en.json";

export type CopyKey = keyof typeof en;

/** Looks up UI text and fills `{name}` placeholders. Add a language by adding a copy file. */
export function t(key: CopyKey, vars: Record<string, string | number> = {}): string {
  return Object.entries(vars).reduce(
    (text, [name, value]) => text.replaceAll(`{${name}}`, String(value)),
    en[key],
  );
}
