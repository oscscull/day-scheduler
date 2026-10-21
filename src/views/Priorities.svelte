<script lang="ts">
  import AreaDot from "../components/AreaDot.svelte";
  import Chip from "../components/Chip.svelte";
  import LoadError from "../components/LoadError.svelte";
  import { api, errorText } from "../lib/api";
  import { t } from "../lib/copy";
  import { ALL, areas, inArea } from "../lib/areas.svelte";
  import { newItem, openItem } from "../lib/detail.svelte";
  import { chatContext } from "../lib/chat.svelte";
  import { workItems } from "../lib/items.svelte";
  import { poll } from "../lib/poll";
  import { reorder } from "../lib/rank";
  import { route } from "../lib/route";
  import type { WorkItem } from "../lib/types";

  const { state: list, load } = workItems();
  let dragging = $state<string | null>(null);
  let over = $state<number | null>(null);
  let saveError = $state("");

  $effect(() => poll(load));

  const inPlay = $derived(list.items.filter((i) => i.stage !== "done" && inArea(i)));

  $effect(() => {
    chatContext.text = JSON.stringify({ area: areas.current, ranked: inPlay });
  });

  // Ids repeat across machines, so the list is keyed by machine and id.
  const keyOf = (i: WorkItem): string => `${i.node ?? ""}|${i.id}`;

  /** Saves the order on the machine that owns `moved`. Each machine ranks its own items. */
  async function save(moved: WorkItem): Promise<void> {
    const same = list.items.filter((i) => i.node === moved.node);
    if (moved.node) {
      await route.moveItem(moved.node, moved.id, same.findIndex((i) => i.id === moved.id) + 1);
    } else {
      await api.reorderItems(same.map((i) => i.id));
    }
  }

  /** Moves `key` to `to` within the visible list. */
  async function move(key: string, to: number): Promise<void> {
    const keys = reorder(list.items.map(keyOf), inPlay.map(keyOf), key, to);
    const byKey = new Map(list.items.map((i) => [keyOf(i), i]));
    list.items = keys.flatMap((k) => byKey.get(k) ?? []);
    const moved = byKey.get(key);
    if (!moved) return;
    try {
      await save(moved);
      saveError = "";
    } catch (e) {
      saveError = t("priorities.saveFailed", { error: errorText(e) });
    }
  }

  function drop(index: number): void {
    const key = dragging;
    dragging = null;
    over = null;
    if (!key) return;
    const from = inPlay.findIndex((i) => keyOf(i) === key);
    void move(key, from < index ? index - 1 : index);
  }

  const showsHolder = (item: WorkItem): string => `${t(`stage.${item.stage}`)} · ${item.holder}`;
</script>

<div class="column">
  <div class="title-row">
    <h1 class="page-title">{t("priorities.title")}</h1>
    <button type="button" class="button-primary" onclick={() => newItem(areas.current !== ALL ? areas.current : (areas.list[0]?.id ?? ""))}>{t("item.new")}</button>
  </div>
  <p class="hint intro">{t("priorities.hint")}</p>
  <LoadError error={list.error} />
  {#if saveError}<p class="error">{saveError}</p>{/if}

  <ol class="list">
    {#each inPlay as item, i (keyOf(item))}
      <li
        draggable="true"
        class:dragging={dragging === keyOf(item)}
        class:over={over === i && dragging !== keyOf(item)}
        ondragstart={(e) => { e.dataTransfer?.setData("text/plain", keyOf(item)); dragging = keyOf(item); }}
        ondragover={(e) => { e.preventDefault(); over = i; }}
        ondrop={(e) => { e.preventDefault(); drop(i); }}
        ondragend={() => { dragging = null; over = null; }}
      >
        <svg class="grip" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="9" cy="6" r="1.6"></circle><circle cx="15" cy="6" r="1.6"></circle><circle cx="9" cy="12" r="1.6"></circle><circle cx="15" cy="12" r="1.6"></circle><circle cx="9" cy="18" r="1.6"></circle><circle cx="15" cy="18" r="1.6"></circle></svg>
        <span class="mono rank">{i + 1}</span>
        <AreaDot area={item.area} />
        <span class="body">
          <button type="button" class="open" onclick={() => openItem(item.id, item.node)}><span class="mono id">{item.id}</span>&nbsp; {item.title}</button>
          <span class="hint meta">{showsHolder(item)}</span>
        </span>
        {#if item.status}<Chip status={item.status} />{/if}
        <span class="arrows">
          <button type="button" aria-label={t("priorities.moveUp", { title: item.title })} onclick={() => move(keyOf(item), i - 1)}>▲</button>
          <button type="button" aria-label={t("priorities.moveDown", { title: item.title })} onclick={() => move(keyOf(item), i + 1)}>▼</button>
        </span>
      </li>
    {:else}
      <li class="hint empty">{t("empty.priorities")}</li>
    {/each}
  </ol>
</div>

<style>
  .title-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .open {
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
  }

  .empty {
    cursor: default;
    padding: 20px 16px;
  }

  .intro {
    margin: 6px 0 28px;
  }

  .list {
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface);
    overflow: hidden;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 12px 12px 8px;
    border-top: 1px solid var(--line-soft);
    cursor: grab;
  }

  li:first-child {
    border-top: 0;
  }

  .dragging {
    background: #f2f0eb;
  }

  .over {
    box-shadow: inset 0 2px 0 var(--accent);
  }

  .grip {
    flex: none;
    color: #9c9991;
  }

  .rank {
    width: 20px;
    flex: none;
    text-align: right;
    font-size: 13px;
    color: var(--muted);
  }

  .body {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .id {
    font-size: 12px;
    color: var(--muted);
  }

  .meta {
    font-size: 13px;
  }

  .arrows {
    display: flex;
    flex-direction: column;
    flex: none;
  }

  .arrows button {
    width: 44px;
    height: 22px;
    border: 0;
    background: transparent;
    color: var(--muted);
    line-height: 1;
  }
</style>
