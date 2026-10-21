<script lang="ts">
  import Modal from "../../components/Modal.svelte";
  import { api, errorText } from "../../lib/api";
  import { t } from "../../lib/copy";

  interface Props {
    onClose: () => void;
    onPaused: () => void;
  }

  let { onClose, onPaused }: Props = $props();

  type Length = "today" | "date" | "resume";

  const isoDate = (d: Date): string =>
    `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  const inAWeek = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);

  let length = $state<Length>("date");
  let date = $state(isoDate(inAWeek));
  let error = $state("");

  /** When the pause ends, as an ISO time; `null` means until resumed. */
  function until(): string | null {
    if (length === "resume") return null;
    if (length === "date") return new Date(`${date}T00:00:00`).toISOString();
    const tomorrow = new Date();
    tomorrow.setHours(24, 0, 0, 0);
    return tomorrow.toISOString();
  }

  async function confirm(): Promise<void> {
    try {
      await api.pauseAll(until());
      onPaused();
    } catch (e) {
      error = t("pause.failed", { error: errorText(e) });
    }
  }
</script>

<Modal title={t("pause.title")} subtitle={t("pause.hint")} {onClose}>
  <fieldset>
    <legend class="sr-only">{t("pause.howLong")}</legend>
    <label><input type="radio" bind:group={length} value="today" /> {t("pause.today")}</label>
    <label>
      <input type="radio" bind:group={length} value="date" /> {t("pause.untilDate")}
      <input type="date" class="input date" bind:value={date} min={isoDate(new Date())} onfocus={() => (length = "date")} />
    </label>
    <label><input type="radio" bind:group={length} value="resume" /> {t("pause.untilResume")}</label>
  </fieldset>
  {#if error}<p class="error">{error}</p>{/if}
  <div class="foot">
    <button type="button" class="button-secondary" onclick={onClose}>{t("pause.cancel")}</button>
    <button type="button" class="button-primary" onclick={confirm}>{t("pause.confirm")}</button>
  </div>
</Modal>

<style>
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  label {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    flex-wrap: wrap;
  }

  .date {
    padding: 6px 10px;
    font-size: 14px;
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
