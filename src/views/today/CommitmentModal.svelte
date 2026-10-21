<script lang="ts">
  import Modal from "../../components/Modal.svelte";
  import { errorText } from "../../lib/api";
  import { ownerOf } from "../../lib/peers.svelte";
  import { route } from "../../lib/route";
  import { ALL, areas } from "../../lib/areas.svelte";
  import { t } from "../../lib/copy";
  import { changed } from "../../lib/refresh.svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  const pad = (n: number): string => String(n).padStart(2, "0");
  const today = new Date();

  let title = $state("");
  let area = $state(areas.current !== ALL ? areas.current : (areas.list[0]?.id ?? ""));
  let date = $state(`${today.getFullYear()}-${pad(today.getMonth() + 1)}-${pad(today.getDate())}`);
  let start = $state("09:00");
  let end = $state("09:30");
  let location = $state("");
  let error = $state("");

  const at = (time: string): string => new Date(`${date}T${time}:00`).toISOString();

  async function save(): Promise<void> {
    try {
      await route.addMeeting(ownerOf(area), {
        title: title.trim(),
        area,
        startsAt: at(start),
        endsAt: at(end),
        location: location.trim() || null,
      });
      changed();
      onClose();
    } catch (e) {
      error = t("commitment.failed", { error: errorText(e) });
    }
  }
</script>

<Modal title={t("commitment.title")} subtitle={t("commitment.calendarHint")} {onClose}>
  <form class="grid" onsubmit={(e) => { e.preventDefault(); void save(); }}>
    <label class="field wide">
      {t("commitment.what")}
      <input class="input" bind:value={title} placeholder={t("commitment.placeholder")} required />
    </label>
    <label class="field">
      {t("item.area")}
      <select class="input" bind:value={area}>
        {#each areas.list as a (a.id)}<option value={a.id}>{a.name}</option>{/each}
      </select>
    </label>
    <label class="field">
      {t("commitment.date")}
      <input class="input" type="date" bind:value={date} />
    </label>
    <label class="field">
      {t("commitment.start")}
      <input class="input" type="time" bind:value={start} />
    </label>
    <label class="field">
      {t("commitment.end")}
      <input class="input" type="time" bind:value={end} />
    </label>
    <label class="field wide">
      {t("commitment.location")}
      <input class="input" bind:value={location} />
    </label>
    {#if error}<p class="error wide">{error}</p>{/if}
    <div class="buttons wide">
      <button type="button" class="button-secondary" onclick={onClose}>{t("commitment.cancel")}</button>
      <button type="submit" class="button-primary" disabled={!title.trim()}>{t("commitment.save")}</button>
    </div>
  </form>
</Modal>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .wide {
    grid-column: 1 / -1;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
