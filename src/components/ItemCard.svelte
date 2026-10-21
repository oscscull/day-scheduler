<script lang="ts">
  import { openItem } from "../lib/detail.svelte";
  import type { WorkItem } from "../lib/types";
  import Age from "./Age.svelte";
  import AreaDot from "./AreaDot.svelte";
  import Chip from "./Chip.svelte";

  interface Props {
    item: WorkItem;
    now: Date;
  }

  let { item, now }: Props = $props();

  // Name the person or system holding it where that adds something.
  const detail = $derived(
    item.status === "review" || item.status === "dependency" ? item.holder : undefined,
  );
</script>

<button type="button" class="card" class:done={item.stage === "done"} onclick={() => openItem(item.id)}>
  <span class="id-row"><AreaDot area={item.area} /><span class="mono id">{item.id}</span></span>
  <span class="title">{item.title}</span>
  <span class="label">{item.stateLabel}</span>
  <span class="foot">
    {#if item.status && item.stage !== "up_next"}
      <Chip status={item.status} {detail} />
    {:else}
      <span class="holder">{item.holder}</span>
    {/if}
    <Age since={item.openedAt} {now} />
  </span>
</button>

<style>
  .card {
    width: 100%;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--line);
  }

  .done {
    background: transparent;
  }

  .id-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .id {
    font-size: 12px;
    color: var(--muted);
  }

  .title {
    font-weight: 500;
    line-height: 1.3;
  }

  .label {
    font-size: 13px;
    color: var(--muted);
    line-height: 1.4;
  }

  .foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-size: 12px;
  }

  .holder {
    color: var(--muted);
  }
</style>
