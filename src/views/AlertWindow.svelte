<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  import { api } from "../lib/api";
  import { chime } from "../lib/chime";
  import { t } from "../lib/copy";
  import { clock } from "../lib/time";
  import type { Commitment } from "../lib/types";

  const POLL_MS = 5_000;
  const TICK_MS = 1_000;

  let meeting = $state<Commitment | null>(null);
  let now = $state(Date.now());
  let chimedLastMinute = false;

  async function load(): Promise<void> {
    meeting = await api.dueAlert().catch(() => null);
  }

  $effect(() => {
    void load().then(chime);
    const poll = setInterval(() => void load(), POLL_MS);
    const tick = setInterval(() => (now = Date.now()), TICK_MS);
    return () => {
      clearInterval(poll);
      clearInterval(tick);
    };
  });

  const seconds = $derived(meeting ? Math.round((new Date(meeting.startsAt).getTime() - now) / 1000) : 0);
  const countdown = $derived(`${Math.floor(Math.max(seconds, 0) / 60)}:${String(Math.max(seconds, 0) % 60).padStart(2, "0")}`);

  // A second chime when a minute is left.
  $effect(() => {
    if (seconds <= 60 && seconds > 0 && !chimedLastMinute) {
      chimedLastMinute = true;
      chime();
    }
  });

  async function join(m: Commitment): Promise<void> {
    if (m.joinUrl) await openUrl(m.joinUrl);
    await api.alertDone(m.id);
  }
</script>

<div class="alert">
  {#if meeting}
    {@const m = meeting}
    <span class="label">{seconds > 0 ? t("alert.startingIn") : t("alert.started")}</span>
    <span class="count mono">{seconds > 0 ? countdown : clock(m.startsAt)}</span>
    <h1>{m.title}</h1>
    <span class="label">{clock(m.startsAt)} – {clock(m.endsAt)}{m.location ? ` · ${m.location}` : ""}</span>
    <div class="spacer"></div>
    <div class="buttons">
      <button type="button" class="join" onclick={() => join(m)}>{t("alert.join")}</button>
      <button type="button" aria-label={t("alert.snoozeLabel")} onclick={() => api.alertSnooze(m.id)}>{t("alert.snooze")}</button>
      <button type="button" onclick={() => api.alertDone(m.id)}>{t("alert.skip")}</button>
    </div>
  {/if}
</div>

<style>
  .alert {
    position: fixed;
    inset: 0;
    box-sizing: border-box;
    padding: 28px;
    background: #191917;
    color: #f5f4f0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    -webkit-user-select: none;
    user-select: none;
  }

  .label {
    font-size: 13px;
    color: #b5b2aa;
  }

  .count {
    font-size: 40px;
    font-weight: 500;
    letter-spacing: -0.02em;
  }

  h1 {
    margin-top: 4px;
    font-size: 22px;
    font-weight: 600;
  }

  .spacer {
    flex: 1 1 0;
  }

  .buttons {
    display: flex;
    gap: 10px;
  }

  button {
    min-height: 44px;
    padding: 0 18px;
    border-radius: 8px;
    border: 1px solid #4a4842;
    background: transparent;
    color: #f5f4f0;
  }

  .join {
    flex: 1 1 0;
    border: 0;
    background: var(--accent);
    font-weight: 600;
  }
</style>
