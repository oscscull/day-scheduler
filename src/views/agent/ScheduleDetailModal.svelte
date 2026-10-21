<script lang="ts">
  import { untrack } from "svelte";

  import Modal from "../../components/Modal.svelte";
  import { api, errorText } from "../../lib/api";
  import { t, type CopyKey } from "../../lib/copy";
  import { dayClock } from "../../lib/time";
  import type { Run, Schedule, Scope } from "../../lib/types";
  import AutonomyChoice from "./AutonomyChoice.svelte";
  import TriggerEditor from "./TriggerEditor.svelte";

  interface Props {
    schedule: Schedule;
    runs: Run[];
    now: Date;
    onClose: () => void;
    onSaved: () => void;
  }

  let { schedule, runs, now, onClose, onSaved }: Props = $props();

  const SCOPES: Scope[] = ["each_pr", "each_item", "one", "nothing"];

  // Edit a copy; nothing changes on disk until Save.
  let form = $state<Schedule>(structuredClone(untrack(() => $state.snapshot(schedule))));
  let saving = $state(false);
  let error = $state("");

  const isNew = $derived(form.id === "");
  const subtitle = $derived(form.request ? t("schedule.from", { request: form.request }) : t("schedule.byHand"));
  const scopeKey = (s: Scope): CopyKey => `scope.${s}`;

  async function save(): Promise<void> {
    saving = true;
    error = "";
    try {
      await api.saveSchedule($state.snapshot(form));
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

  <TriggerEditor trigger={form.trigger} {now} onChange={(next) => (form.trigger = next)} />

  <div class="pair">
    <label class="field">
      {t("schedule.node")}
      <input class="input" bind:value={form.node} />
    </label>
    <label class="field">
      {t("schedule.scope")}
      <select class="input" bind:value={form.scope}>
        {#each SCOPES as s (s)}<option value={s}>{t(scopeKey(s))}</option>{/each}
      </select>
    </label>
  </div>

  <AutonomyChoice value={form.autonomy} onChange={(v) => (form.autonomy = v)} />

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

  .pair {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
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
