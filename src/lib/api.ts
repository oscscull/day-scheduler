import { invoke } from "@tauri-apps/api/core";

import type { AgentView, Explanation, Schedule, TodayView, WorkItem } from "./types";

/** Typed wrappers around the backend's Tauri commands. */
export const api = {
  today: () => invoke<TodayView>("today"),
  workItems: () => invoke<WorkItem[]>("work_items"),
  agent: () => invoke<AgentView>("agent"),
  reorderItems: (ids: string[]) => invoke<undefined>("reorder_items", { ids }),
  setScheduleEnabled: (id: string, enabled: boolean) =>
    invoke<undefined>("set_schedule_enabled", { id, enabled }),
  saveSchedule: (schedule: Schedule) => invoke<string>("save_schedule", { schedule }),
  deleteSchedule: (id: string) => invoke<undefined>("delete_schedule", { id }),
  explainCron: (expr: string) => invoke<Explanation>("explain_cron", { expr }),
  askClaude: (page: string, context: string, message: string) =>
    invoke<string>("ask_claude", { page, context, message }),
  draftSchedule: (request: string) => invoke<string>("draft_schedule", { request }),
};

/** Backend errors arrive as plain strings. */
export function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
