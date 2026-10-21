import { invoke } from "@tauri-apps/api/core";

import type {
  AgentView,
  Area,
  Commitment,
  CommitmentInput,
  Explanation,
  ItemDetail,
  ItemInput,
  Schedule,
  TodayView,
  WorkItem,
} from "./types";

/** Typed wrappers around the backend's Tauri commands. */
export const api = {
  mcpConnectCommand: () => invoke<string>("mcp_connect_command"),
  dueAlert: () => invoke<Commitment | null>("due_alert"),
  alertDone: (id: number) => invoke<undefined>("alert_done", { id }),
  alertSnooze: (id: number) => invoke<undefined>("alert_snooze", { id }),
  areas: () => invoke<Area[]>("areas"),
  addTodo: (title: string, area: string, due: string | null) =>
    invoke<undefined>("add_todo", { title, area, due }),
  setTodoDone: (id: number, done: boolean) => invoke<undefined>("set_todo_done", { id, done }),
  today: () => invoke<TodayView>("today"),
  workItems: () => invoke<WorkItem[]>("work_items"),
  agent: () => invoke<AgentView>("agent"),
  saveItem: (item: ItemInput) => invoke<string>("save_item", { item }),
  deleteItem: (id: string) => invoke<undefined>("delete_item", { id }),
  itemDetail: (id: string) => invoke<ItemDetail>("item_detail", { id }),
  addAsk: (itemId: string, question: string, context: string) =>
    invoke<undefined>("add_ask", { itemId, question, context }),
  resolveAsk: (id: number) => invoke<undefined>("resolve_ask", { id }),
  addCommitment: (commitment: CommitmentInput) => invoke<undefined>("add_commitment", { commitment }),
  hideCommitment: (id: number) => invoke<undefined>("hide_commitment", { id }),
  deleteCommitment: (id: number) => invoke<undefined>("delete_commitment", { id }),
  reorderItems: (ids: string[]) => invoke<undefined>("reorder_items", { ids }),
  setScheduleEnabled: (id: string, enabled: boolean) =>
    invoke<undefined>("set_schedule_enabled", { id, enabled }),
  saveSchedule: (schedule: Schedule) => invoke<string>("save_schedule", { schedule }),
  deleteSchedule: (id: string) => invoke<undefined>("delete_schedule", { id }),
  explainCron: (expr: string) => invoke<Explanation>("explain_cron", { expr }),
  runScheduleNow: (id: string) => invoke<undefined>("run_schedule_now", { id }),
  pauseAll: (until: string | null) => invoke<undefined>("pause_all", { until }),
  resumeAll: () => invoke<undefined>("resume_all"),
  dismissUrgent: (id: number) => invoke<undefined>("dismiss_urgent", { id }),
  dismissAllUrgent: () => invoke<undefined>("dismiss_all_urgent"),
  resolveUrgent: (id: number, message: string) => invoke<string>("resolve_urgent", { id, message }),
  askClaude: (page: string, context: string, message: string) =>
    invoke<string>("ask_claude", { page, context, message }),
  draftSchedule: (request: string) => invoke<string>("draft_schedule", { request }),
};

/** Backend errors arrive as plain strings. */
export function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
