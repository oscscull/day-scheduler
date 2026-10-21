<script lang="ts">
  import AreaDot from "../../components/AreaDot.svelte";
  import { api, errorText } from "../../lib/api";
  import { areas } from "../../lib/areas.svelte";
  import { t } from "../../lib/copy";
  import { changed } from "../../lib/refresh.svelte";
  import type { Calendar } from "../../lib/types";

  interface Props {
    /** Sync state for the Calendar row, already worded. */
    syncText: string;
    failed: boolean;
  }

  let { syncText, failed }: Props = $props();

  let calendars = $state<Calendar[]>([]);
  let adding = $state(false);
  let removing = $state<string | null>(null);
  let url = $state("");
  let area = $state("");
  let error = $state("");

  async function load(): Promise<void> {
    calendars = await api.calendars().catch(() => []);
  }

  $effect(() => {
    void load();
  });

  function open(): void {
    adding = true;
    area = areas.list[0]?.id ?? "";
  }

  async function act(work: () => Promise<unknown>): Promise<void> {
    try {
      await work();
      error = "";
      adding = false;
      removing = null;
      url = "";
      await load();
      changed();
    } catch (e) {
      error = t("calendar.failed", { error: errorText(e) });
    }
  }
</script>

<li class="calendar">
  <div>
    <span class="name">{t("connections.calendar")}</span>
    <span class:error={failed && calendars.length > 0} class="hint">
      {calendars.length === 0 ? t("connections.notSetUp") : syncText}
    </span>
  </div>

  {#each calendars as c (c.name)}
    <div class="feed">
      <AreaDot area={c.area} />
      <span>{c.name}</span>
      {#if removing === c.name}
        <span class="hint">{t("calendar.confirmRemove", { name: c.name })}</span>
        <button type="button" class="text-button danger" onclick={() => act(() => api.removeCalendar(c.name))}>{t("calendar.removeYes")}</button>
        <button type="button" class="text-button" onclick={() => (removing = null)}>{t("calendar.keep")}</button>
      {:else}
        <button type="button" class="text-button" aria-label={t("calendar.remove", { name: c.name })} onclick={() => (removing = c.name)}>×</button>
      {/if}
    </div>
  {/each}

  {#if adding}
    <form onsubmit={(e) => { e.preventDefault(); void act(() => api.addCalendar(url, area)); }}>
      <label>
        <span class="section-label">{t("calendar.link")}</span>
        <input class="input" bind:value={url} placeholder="https://… .ics" required />
      </label>
      <p class="hint small">{t("calendar.linkHint")}</p>
      {#if areas.list.length > 1}
        <label>
          <span class="section-label">{t("calendar.area")}</span>
          <select class="input" bind:value={area}>
            {#each areas.list as a (a.id)}<option value={a.id}>{a.name}</option>{/each}
          </select>
        </label>
      {/if}
      <div class="actions">
        <button type="button" class="button-secondary" onclick={() => (adding = false)}>{t("calendar.cancel")}</button>
        <button type="submit" class="button-primary" disabled={!url.trim()}>{t("calendar.save")}</button>
      </div>
    </form>
  {:else}
    <button type="button" class="text-button add" onclick={open}>
      {calendars.length === 0 ? t("calendar.add") : t("calendar.addAnother")}
    </button>
  {/if}
  {#if error}<p class="error small">{error}</p>{/if}
</li>

<style>
  .calendar {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .name {
    font-weight: 500;
  }

  .hint {
    font-size: 13px;
  }

  .small {
    font-size: 12px;
  }

  .feed {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 12px;
    font-size: 13px;
  }

  .text-button {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    font-size: 13px;
    color: var(--muted);
    cursor: pointer;
  }

  .text-button:hover {
    color: var(--ink);
  }

  .add {
    align-self: flex-start;
    color: var(--accent);
  }

  .danger {
    color: var(--urgent-fg);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 520px;
    padding: 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .error {
    color: var(--urgent-fg);
  }
</style>
