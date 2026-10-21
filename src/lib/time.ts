import { t } from "./copy";

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** Items open longer than this are shown as stale. */
export const STALE_AFTER = 3 * DAY;

const clockFormat = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const weekdayFormat = new Intl.DateTimeFormat(undefined, { weekday: "short" });
const longDateFormat = new Intl.DateTimeFormat(undefined, {
  weekday: "long",
  day: "numeric",
  month: "long",
});

export const clock = (iso: string): string => clockFormat.format(new Date(iso));
export const longDate = (date: Date): string => longDateFormat.format(date);

const sameDay = (a: Date, b: Date): boolean => a.toDateString() === b.toDateString();

/** "16:00" today, otherwise "Thu 09:30". */
export function dayClock(iso: string, now: Date): string {
  const date = new Date(iso);
  return sameDay(date, now) ? clock(iso) : `${weekdayFormat.format(date)} ${clock(iso)}`;
}

/** "in 28 min", "in 2h 5m", or "now" once started. */
export function until(iso: string, now: Date): string {
  const ms = new Date(iso).getTime() - now.getTime();
  if (ms <= 0) return t("time.now");
  const minutes = Math.ceil(ms / MINUTE);
  if (minutes < 60) return t("time.inMinutes", { n: minutes });
  return t("time.inHours", { h: Math.floor(minutes / 60), m: minutes % 60 });
}

export function ageMs(iso: string, now: Date): number {
  return now.getTime() - new Date(iso).getTime();
}

/** Compact age: "42m", "2h", "9d". */
export function age(iso: string, now: Date): string {
  const ms = Math.max(0, ageMs(iso, now));
  if (ms < HOUR) return t("age.minutes", { n: Math.floor(ms / MINUTE) });
  if (ms < DAY) return t("age.hours", { n: Math.floor(ms / HOUR) });
  return t("age.days", { n: Math.floor(ms / DAY) });
}

export const isStale = (iso: string, now: Date): boolean => ageMs(iso, now) >= STALE_AFTER;
