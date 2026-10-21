<script lang="ts">
  import { ALL, areas, selectArea } from "../lib/areas.svelte";
  import { t } from "../lib/copy";

  const options = $derived([
    { id: ALL, name: t("area.all"), color: "" },
    ...areas.list.map((a) => ({ id: a.id, name: a.name, color: a.color })),
  ]);
</script>

<nav aria-label={t("area.label")}>
  {#each options as o (o.id)}
    <button type="button" class:active={areas.current === o.id} aria-pressed={areas.current === o.id} onclick={() => selectArea(o.id)}>
      {#if o.color}<span class="dot" style:background={o.color}></span>{/if}{o.name}
    </button>
  {/each}
</nav>

<style>
  nav {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 2px;
  }

  button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 10px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
  }

  .active {
    background: var(--surface);
    color: var(--ink);
    font-weight: 500;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }
</style>
