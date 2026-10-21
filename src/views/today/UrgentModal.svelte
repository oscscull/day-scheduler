<script lang="ts">
  import Modal from "../../components/Modal.svelte";
  import { api, errorText } from "../../lib/api";
  import { route } from "../../lib/route";
  import { t } from "../../lib/copy";
  import type { Urgent } from "../../lib/types";
  import UrgentRow from "./UrgentRow.svelte";

  interface Props {
    items: Urgent[];
    now: Date;
    onClose: () => void;
    onChanged: () => void;
  }

  let { items, now, onClose, onChanged }: Props = $props();

  let error = $state("");

  async function dismissAll(): Promise<void> {
    try {
      await api.dismissAllUrgent();
      await Promise.all(items.filter((i) => i.node).map((i) => route.dismissUrgent(i.node, i.id)));
      onChanged();
      onClose();
    } catch (e) {
      error = t("urgent.failed", { error: errorText(e) });
    }
  }
</script>

<Modal title={t("urgent.title")} subtitle={t("urgent.hint")} {onClose}>
  <ul>
    {#each items as item (`${item.node ?? ""}${item.id}`)}
      <UrgentRow {item} {now} onDone={onChanged} />
    {:else}
      <li class="row hint clear">{t("urgent.allClear")}</li>
    {/each}
  </ul>
  {#if error}<p class="error">{error}</p>{/if}
  {#if items.length > 0}
    <div class="foot">
      <button type="button" class="link" onclick={dismissAll}>{t("urgent.dismissAll")}</button>
    </div>
  {/if}
</Modal>

<style>
  .clear {
    padding: 20px 0;
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--line);
  }

  .link {
    padding: 10px 0;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
    text-decoration: underline;
  }
</style>
