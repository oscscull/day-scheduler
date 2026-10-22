// Puts the running version in the title bar, so it's clear when an update has landed.
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { t } from "./copy";

export async function showVersion(): Promise<void> {
  try {
    await getCurrentWindow().setTitle(t("app.title", { version: await getVersion() }));
  } catch {
    // The title is a courtesy; the plain one stays.
  }
}
