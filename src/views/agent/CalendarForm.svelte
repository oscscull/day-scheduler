<script lang="ts">
  import { api, errorText } from "../../lib/api";
  import { areas } from "../../lib/areas.svelte";
  import { t } from "../../lib/copy";
  import type { MacCalendar } from "../../lib/types";

  interface Props {
    /** Runs the save, then closes the form or shows its error. */
    onSave: (work: () => Promise<unknown>) => void;
    onCancel: () => void;
  }

  let { onSave, onCancel }: Props = $props();

  type Mode = "mac" | "link";
  let mode = $state<Mode>("mac");
  let url = $state("");
  let area = $state(areas.list[0]?.id ?? "");
  let calendar = $state("");
  let macCalendars = $state<MacCalendar[] | null>(null);
  let macError = $state("");

  /** A title, or "account/title" when another account has a calendar with the same title. */
  function nameOf(c: MacCalendar, all: MacCalendar[]): string {
    const shared = all.filter((o) => o.title === c.title).length > 1;
    return shared ? `${c.account}/${c.title}` : c.title;
  }

  // Asking for the list is what has macOS ask for calendar access, so only do it in this mode.
  $effect(() => {
    if (mode !== "mac" || macCalendars !== null) return;
    api
      .macCalendars()
      .then((list) => {
        macCalendars = list;
        calendar = list[0] ? nameOf(list[0], list) : "";
      })
      .catch((e: unknown) => {
        macCalendars = [];
        macError = t("calendar.macFailed", { error: errorText(e) });
      });
  });

  const ready = $derived(mode === "mac" ? calendar !== "" : url.trim() !== "");

  function save(): void {
    onSave(() => (mode === "mac" ? api.addMacCalendar(calendar, area) : api.addCalendar(url, area)));
  }
</script>

<form onsubmit={(e) => { e.preventDefault(); save(); }}>
  <div class="modes" role="group" aria-label={t("calendar.source")}>
    <button type="button" class:active={mode === "mac"} aria-pressed={mode === "mac"} onclick={() => (mode = "mac")}>
      {t("calendar.fromMac")}
    </button>
    <button type="button" class:active={mode === "link"} aria-pressed={mode === "link"} onclick={() => (mode = "link")}>
      {t("calendar.fromLink")}
    </button>
  </div>

  {#if mode === "mac"}
    {#if macCalendars === null}
      <p class="hint small">{t("calendar.macLoading")}</p>
    {:else if macError}
      <p class="error small">{macError}</p>
    {:else if macCalendars.length === 0}
      <p class="hint small">{t("calendar.macEmpty")}</p>
    {:else}
      <label>
        <span class="section-label">{t("calendar.macCalendar")}</span>
        <select class="input" bind:value={calendar}>
          {#each macCalendars as c (`${c.account}/${c.title}`)}
            <option value={nameOf(c, macCalendars)}>{t("calendar.macOption", { title: c.title, account: c.account })}</option>
          {/each}
        </select>
      </label>
    {/if}
    <p class="hint small">{t("calendar.macHint")}</p>
  {:else}
    <label>
      <span class="section-label">{t("calendar.link")}</span>
      <input class="input" bind:value={url} placeholder="https://… .ics" required />
    </label>
    <p class="hint small">{t("calendar.linkHint")}</p>
  {/if}

  {#if areas.list.length > 1}
    <label>
      <span class="section-label">{t("calendar.area")}</span>
      <select class="input" bind:value={area}>
        {#each areas.list as a (a.id)}<option value={a.id}>{a.name}</option>{/each}
      </select>
    </label>
  {/if}
  <div class="actions">
    <button type="button" class="button-secondary" onclick={onCancel}>{t("calendar.cancel")}</button>
    <button type="submit" class="button-primary" disabled={!ready}>{t("calendar.save")}</button>
  </div>
</form>

<style>
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

  .modes {
    display: flex;
    align-self: flex-start;
    gap: 2px;
    padding: 3px;
    border-radius: 9px;
    background: var(--switch-track);
  }

  .modes button {
    padding: 5px 12px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
  }

  .modes .active {
    background: var(--surface);
    color: var(--ink);
    font-weight: 500;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hint {
    font-size: 13px;
  }

  .small {
    font-size: 12px;
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
