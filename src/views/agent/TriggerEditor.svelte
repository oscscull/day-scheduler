<script lang="ts">
  import { untrack } from "svelte";

  import { t } from "../../lib/copy";
  import type { EventKind, Trigger } from "../../lib/types";
  import CronExplain from "./CronExplain.svelte";

  interface Props {
    trigger: Trigger;
    now: Date;
    onChange: (trigger: Trigger) => void;
  }

  let { trigger, now, onChange }: Props = $props();

  const EVENTS: EventKind[] = ["pr_new_comments", "ci_failed", "ci_passed"];

  // Keep both shapes so switching back and forth doesn't lose what was typed.
  const start = untrack(() => trigger);
  let mode = $state(start.type);
  let expr = $state(start.type === "cron" ? start.expr : "0 9 * * 1-5");
  let kind = $state<EventKind>(start.type === "event" ? start.kind : "pr_new_comments");

  $effect(() => {
    onChange(mode === "cron" ? { type: "cron", expr } : { type: "event", kind });
  });
</script>

<fieldset>
  <legend class="field">{t("schedule.trigger")}</legend>
  <div class="modes">
    <label><input type="radio" bind:group={mode} value="cron" /> {t("schedule.onSchedule")}</label>
    <label><input type="radio" bind:group={mode} value="event" /> {t("schedule.onEvent")}</label>
  </div>

  {#if mode === "cron"}
    <label class="field">
      {t("schedule.cron")}
      <input class="input mono" bind:value={expr} spellcheck="false" />
    </label>
    <CronExplain {expr} {now} />
  {:else}
    <label class="field">
      {t("schedule.event")}
      <select class="input" bind:value={kind}>
        {#each EVENTS as e (e)}<option value={e}>{t(`event.${e}`)}</option>{/each}
      </select>
    </label>
    <p class="hint small">{t("schedule.eventHint")}</p>
  {/if}
</fieldset>

<style>
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  legend {
    margin-bottom: 8px;
  }

  .modes {
    display: flex;
    gap: 20px;
    font-size: 14px;
  }

  .small {
    font-size: 13px;
  }
</style>
