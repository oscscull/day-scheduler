<script lang="ts">
  import Modal from "../../components/Modal.svelte";
  import { api, errorText } from "../../lib/api";
  import { t, type CopyKey } from "../../lib/copy";

  interface Props {
    onClose: () => void;
    onCreated: (id: string) => void;
    onManual: () => void;
  }

  let { onClose, onCreated, onManual }: Props = $props();

  const EXAMPLES: CopyKey[] = ["newSchedule.example1", "newSchedule.example2", "newSchedule.example3"];

  let request = $state("");
  let busy = $state(false);
  let error = $state("");

  async function create(): Promise<void> {
    if (!request.trim() || busy) return;
    busy = true;
    error = "";
    try {
      onCreated(await api.draftSchedule(request.trim()));
    } catch (e) {
      error = t("newSchedule.failed", { error: errorText(e) });
    } finally {
      busy = false;
    }
  }
</script>

<Modal title={t("newSchedule.title")} {onClose}>
  <label class="field">
    <span class="hint">{t("newSchedule.hint")}</span>
    <textarea rows="5" bind:value={request} placeholder={t("newSchedule.placeholder")} disabled={busy}></textarea>
  </label>
  <div class="examples">
    {#each EXAMPLES as key (key)}
      <button type="button" onclick={() => (request = t(key))} disabled={busy}>{t(key)}</button>
    {/each}
  </div>
  {#if busy}<p class="hint">{t("newSchedule.working")}</p>{/if}
  {#if error}<p class="error">{error}</p>{/if}
  <div class="foot">
    <button type="button" class="link" onclick={onManual} disabled={busy}>{t("newSchedule.manual")}</button>
    <button type="button" class="button-primary" onclick={create} disabled={busy || !request.trim()}>
      {t("newSchedule.create")}
    </button>
  </div>
</Modal>

<style>
  textarea {
    padding: 14px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--surface-soft);
    color: var(--ink);
    font-size: 16px;
    line-height: 1.5;
    resize: vertical;
  }

  .examples {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .examples button {
    padding: 7px 12px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--surface);
    color: #4a4843;
    font-size: 13px;
  }

  .foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
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
