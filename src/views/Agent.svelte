<script lang="ts">
  import LoadError from "../components/LoadError.svelte";
  import { api, errorText } from "../lib/api";
  import { chatContext } from "../lib/chat.svelte";
  import { t } from "../lib/copy";
  import { poll } from "../lib/poll";
  import type { AgentView, Schedule } from "../lib/types";
  import DeleteScheduleModal from "./agent/DeleteScheduleModal.svelte";
  import LiveRuns from "./agent/LiveRuns.svelte";
  import NewScheduleModal from "./agent/NewScheduleModal.svelte";
  import ScheduleDetailModal from "./agent/ScheduleDetailModal.svelte";
  import ScheduleList from "./agent/ScheduleList.svelte";
  import { blankSchedule } from "./agent/blank";

  let data = $state<AgentView | null>(null);
  let error = $state("");
  let now = $state(new Date());
  let creating = $state(false);
  let editing = $state<Schedule | null>(null);
  let deleting = $state<Schedule | null>(null);

  async function load(): Promise<void> {
    try {
      data = await api.agent();
      now = new Date();
      error = "";
      chatContext.text = JSON.stringify(data);
    } catch (e) {
      error = errorText(e);
    }
  }

  $effect(() => poll(load));

  async function setEnabled(id: string, enabled: boolean): Promise<void> {
    try {
      await api.setScheduleEnabled(id, enabled);
    } catch (e) {
      error = errorText(e);
    }
    await load();
  }

  /** After Claude drafts a schedule, open it so the user can check it. */
  async function created(id: string): Promise<void> {
    creating = false;
    await load();
    editing = data?.schedules.find((s) => s.id === id) ?? null;
  }

  const editingView = $derived(data?.schedules.find((s) => s.id === editing?.id) ?? null);
</script>

<div class="column">
  <LoadError {error} />
  {#if data}
    <LiveRuns runs={data.live} {now} />

    <section class="scheduled" aria-label={t("agent.scheduled")}>
      <div class="head">
        <h2>{t("agent.scheduled")}</h2>
        <button type="button" class="button-primary" onclick={() => (creating = true)}>{t("agent.new")}</button>
      </div>
      {#if data.problems.length > 0}
        <div class="problems" role="alert">
          <p>{t("agent.problems")}</p>
          <ul>{#each data.problems as problem (problem)}<li class="mono">{problem}</li>{/each}</ul>
        </div>
      {/if}
      <ScheduleList schedules={data.schedules} {now} onOpen={(s) => (editing = s)} onToggle={setEnabled} onDelete={(s) => (deleting = s)} />
    </section>
  {/if}
</div>

{#if creating}
  <NewScheduleModal
    onClose={() => (creating = false)}
    onCreated={created}
    onManual={() => { creating = false; editing = blankSchedule(); }}
  />
{/if}

{#if deleting}
  <DeleteScheduleModal
    schedule={deleting}
    onClose={() => (deleting = null)}
    onDeleted={() => { deleting = null; void load(); }}
  />
{/if}

{#if editing}
  <ScheduleDetailModal
    schedule={editing}
    runs={editingView?.runs ?? []}
    {now}
    onClose={() => (editing = null)}
    onSaved={() => { editing = null; void load(); }}
  />
{/if}

<style>
  .scheduled {
    margin-top: 44px;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .problems {
    margin-bottom: 12px;
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--chip-decision-bg);
    color: var(--chip-decision-fg);
    font-size: 13px;
  }

  .problems li {
    margin-top: 4px;
    font-size: 12px;
  }
</style>
