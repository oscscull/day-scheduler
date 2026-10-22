<script lang="ts">
  import { api, errorText } from "../../lib/api";
  import { t } from "../../lib/copy";
  import type { MacCalendar } from "../../lib/types";

  interface Props {
    /** The picked calendar's name as the feed stores it; empty until there is one. */
    calendar: string;
  }

  let { calendar = $bindable() }: Props = $props();

  let calendars = $state<MacCalendar[] | null>(null);
  let error = $state("");

  /** A title, or "account/title" when another account has a calendar with the same title. */
  function nameOf(c: MacCalendar, all: MacCalendar[]): string {
    const shared = all.filter((o) => o.title === c.title).length > 1;
    return shared ? `${c.account}/${c.title}` : c.title;
  }

  // Asking for the list is what has macOS ask for calendar access.
  $effect(() => {
    api
      .macCalendars()
      .then((list) => {
        calendars = list;
        calendar = list[0] ? nameOf(list[0], list) : "";
      })
      .catch((e: unknown) => {
        calendars = [];
        error = t("calendar.macFailed", { error: errorText(e) });
      });
  });
</script>

{#if calendars === null}
  <p class="hint">{t("calendar.macLoading")}</p>
{:else if error}
  <p class="hint error">{error}</p>
{:else if calendars.length === 0}
  <p class="hint">{t("calendar.macEmpty")}</p>
{:else}
  <label>
    <span class="section-label">{t("calendar.macCalendar")}</span>
    <select class="input" bind:value={calendar}>
      {#each calendars as c (`${c.account}/${c.title}`)}
        <option value={nameOf(c, calendars)}>{t("calendar.macOption", { title: c.title, account: c.account })}</option>
      {/each}
    </select>
  </label>
{/if}
<p class="hint">{t("calendar.macHint")}</p>

<style>
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hint {
    font-size: 12px;
  }

  .error {
    color: var(--urgent-fg);
  }
</style>
