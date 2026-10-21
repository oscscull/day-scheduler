<script lang="ts">
  import Modal from "../../components/Modal.svelte";
  import { api, errorText } from "../../lib/api";
  import { t } from "../../lib/copy";
  import type { Schedule } from "../../lib/types";

  interface Props {
    schedule: Schedule;
    onClose: () => void;
    onDeleted: () => void;
  }

  let { schedule, onClose, onDeleted }: Props = $props();

  let busy = $state(false);
  let error = $state("");

  async function confirm(): Promise<void> {
    busy = true;
    error = "";
    try {
      await api.deleteSchedule(schedule.id);
      onDeleted();
    } catch (e) {
      error = t("deleteSchedule.failed", { error: errorText(e) });
    } finally {
      busy = false;
    }
  }
</script>

<Modal title={t("deleteSchedule.title")} {onClose}>
  <p class="hint">{t("deleteSchedule.body", { name: schedule.name, id: schedule.id })}</p>
  {#if error}<p class="error">{error}</p>{/if}
  <div class="foot">
    <button type="button" class="button-secondary" onclick={onClose}>{t("deleteSchedule.cancel")}</button>
    <button type="button" class="danger" onclick={confirm} disabled={busy}>{t("deleteSchedule.confirm")}</button>
  </div>
</Modal>

<style>
  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .danger {
    padding: 10px 16px;
    border: 0;
    border-radius: 8px;
    background: var(--chip-decision-fg);
    color: #fff;
    font-size: 14px;
    font-weight: 500;
  }

  .danger:disabled {
    opacity: 0.5;
  }
</style>
