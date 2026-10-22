<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  import { errorText } from "../../lib/api";
  import { route } from "../../lib/route";
  import { findArea } from "../../lib/areas.svelte";
  import { t } from "../../lib/copy";
  import { editItem } from "../../lib/detail.svelte";
  import { changed, refresh } from "../../lib/refresh.svelte";
  import { age } from "../../lib/time";
  import type { ItemDetail } from "../../lib/types";
  import Chip from "../Chip.svelte";
  import Modal from "../Modal.svelte";
  import AskList from "./AskList.svelte";
  import TellClaude from "../TellClaude.svelte";
  import History from "./History.svelte";
  import Links from "./Links.svelte";

  interface Props {
    id: string;
    /** The machine it lives on; unset for this one. */
    node: string | undefined;
    onClose: () => void;
  }

  let { id, node, onClose }: Props = $props();

  let detail = $state<ItemDetail | null>(null);
  let error = $state("");
  let confirming = $state(false);
  const now = new Date();

  // Reload whenever anything is edited, including asks added here.
  $effect(() => {
    void refresh.version;
    route.itemDetail(node, id).then((d) => (detail = { ...d, item: { ...d.item, ...(node ? { node } : {}) } })).catch((e: unknown) => (error = errorText(e)));
  });

  async function park(parked: boolean): Promise<void> {
    try {
      await route.setParked(node, id, parked);
      changed();
      if (parked) onClose();
    } catch (e) {
      error = errorText(e);
    }
  }

  async function remove(): Promise<void> {
    try {
      await route.deleteItem(node, id);
      changed();
      onClose();
    } catch (e) {
      error = errorText(e);
    }
  }

  const subtitle = $derived(
    detail
      ? t("item.since", {
          state: t(`stage.${detail.item.stage}`),
          age: age(detail.item.stateSince, now),
          open: age(detail.item.openedAt, now),
        })
      : "",
  );
</script>

<Modal title={detail ? `${detail.item.id} · ${detail.item.title}` : id} {subtitle} {onClose}>
  {#if detail}
    {@const item = detail.item}
    <div class="state">
      {#if findArea(item.area)}<span class="area"><span class="dot" style:background={findArea(item.area)?.color}></span>{findArea(item.area)?.name}</span>{/if}
      {#if item.status}<Chip status={item.status} />{/if}
      {#if item.stateLabel}<span class="label">{item.stateLabel}</span>{/if}
      {#if item.holder}<span class="hint">· {item.holder}</span>{/if}
    </div>
    {#if item.link}<Links link={item.link} />{/if}
    {#if item.source !== "manual"}<p class="hint small">{t("item.synced", { source: item.source })}</p>{/if}
    {#if detail.parked}<p class="hint small">{t("item.parked")}</p>{/if}
    <AskList {node} asks={detail.asks} />
    {#if item.stage !== "done"}
      <section class="tell" aria-label={t("tell.label")}>
        <h3 class="section-label">{t("tell.label")}</h3>
        <TellClaude itemId={item.id} {node} />
      </section>
    {/if}
    <History events={detail.history} {now} />
    {#if confirming}
      <p class="error">{t("item.confirmDelete", { id: item.id })}</p>
    {/if}
    <div class="foot">
      <button type="button" class="quiet" onclick={() => (confirming ? remove() : (confirming = true))}>{t("item.delete")}</button>
      <div class="buttons">
        {#if detail.parked !== undefined && item.stage !== "done"}
          {@const parked = detail.parked}
          <button type="button" class="button-secondary" onclick={() => park(!parked)}>{parked ? t("item.unpark") : t("item.park")}</button>
        {/if}
        {#if item.link}
          {@const link = item.link}
          <button type="button" class="button-secondary" onclick={() => openUrl(link)}>{t("item.open")}</button>
        {/if}
        <button type="button" class="button-primary" onclick={() => { editItem(item); onClose(); }}>{t("item.edit")}</button>
      </div>
    </div>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
</Modal>

<style>
  .small {
    font-size: 13px;
  }

  .state {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .area {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }

  .foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .quiet {
    padding: 10px 0;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
  }
  .tell {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
