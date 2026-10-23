<script lang="ts">
  import { untrack } from "svelte";

  import Modal from "../../components/Modal.svelte";
  import { areas, findArea } from "../../lib/areas.svelte";
  import { errorText } from "../../lib/api";
  import { route } from "../../lib/route";
  import { t } from "../../lib/copy";
  import { dayClock } from "../../lib/time";
  import type { Run, Schedule } from "../../lib/types";
  import TriggersEditor from "./TriggersEditor.svelte";

  interface Props {
    schedule: Schedule;
    runs: Run[];
    now: Date;
    onClose: () => void;
    onSaved: () => void;
  }

  let { schedule, runs, now, onClose, onSaved }: Props = $props();

  // Edit a copy; nothing changes on disk until Save.
  let form = $state<Schedule>(structuredClone(untrack(() => $state.snapshot(schedule))));
  let saving = $state(false);
  let error = $state("");

  const isNew = $derived(form.id === "");
  const subtitle = $derived(form.request ? t("schedule.from", { request: form.request }) : t("schedule.byHand"));

  async function runNow(): Promise<void> {
    try {
      await route.runSchedule(form.node, form.id);
      onSaved();
    } catch (e) {
      error = errorText(e);
    }
  }

  async function save(): Promise<void> {
    saving = true;
    error = "";
    try {
      await route.saveSchedule(form.node, $state.snapshot(form));
      onSaved();
    } catch (e) {
      error = t("schedule.saveFailed", { error: errorText(e) });
    } finally {
      saving = false;
    }
  }
</script>

<Modal title={form.name} {subtitle} {onClose}>
  <label class="field">
    {t("schedule.name")}
    <input class="input" bind:value={form.name} />
  </label>

  <label class="field">
    {t("schedule.instructions")}
    <textarea class="input" rows="4" bind:value={form.prompt}></textarea>
  </label>

  <TriggersEditor triggers={form.triggers} {now} onChange={(next) => (form.triggers = next)} />

  <label class="field">
    {t("schedule.area")}
    <select class="input" bind:value={form.area}>
      {#each areas.list as a (a.id)}<option value={a.id}>{a.name}</option>{/each}
    </select>
    {#if findArea(form.area)}<span class="hint small">{t("schedule.runsOn", { node: findArea(form.area)?.node ?? "" })}</span>{/if}
  </label>

  {#if runs.length > 0}
    <div>
      <h3 class="section-label">{t("schedule.recent")}</h3>
      <ul>
        {#each runs as run (run.id)}
          <li class="run">
            <span class="mono when">{dayClock(run.startedAt, now)}</span>
            <span>{run.summary ?? run.what}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}

  <div class="foot">
    <span class="mono file">{isNew ? t("schedule.newFile") : t("schedule.file", { id: form.id })}</span>
    <div class="buttons">
      {#if !isNew}
        <button type="button" class="button-secondary" onclick={runNow}>{t("schedule.runNow")}</button>
      {/if}
      <button type="button" class="button-secondary" onclick={onClose}>{t("schedule.cancel")}</button>
      <button type="button" class="button-primary" onclick={save} disabled={saving || !form.name.trim()}>
        {t("schedule.save")}
      </button>
    </div>
  </div>
</Modal>

<style>
  textarea {
    line-height: 1.45;
    resize: vertical;
    background: var(--surface-soft);
  }

  .small {
    font-size: 12px;
  }

  .run {
    display: flex;
    gap: 12px;
    padding: 8px 0;
    border-top: 1px solid var(--line-soft);
    font-size: 14px;
  }

  .when {
    width: 84px;
    flex: none;
    font-size: 13px;
    color: var(--muted);
  }

  h3 {
    margin-bottom: 6px;
  }

  .foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }

  .file {
    font-size: 12px;
    color: var(--muted);
  }

  .buttons {
    display: flex;
    gap: 8px;
  }
</style>
