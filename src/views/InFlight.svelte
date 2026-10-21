<script lang="ts">
  import Age from "../components/Age.svelte";
  import AreaDot from "../components/AreaDot.svelte";
  import Chip from "../components/Chip.svelte";
  import LoadError from "../components/LoadError.svelte";
  import { t } from "../lib/copy";
  import { WAITING, isInFlight, waitingOn } from "../lib/groups";
  import { areas, inArea } from "../lib/areas.svelte";
  import { chatContext } from "../lib/chat.svelte";
  import { openItem } from "../lib/detail.svelte";
  import { workItems } from "../lib/items.svelte";
  import { poll } from "../lib/poll";
  import { age } from "../lib/time";

  const { state: list, load } = workItems();

  $effect(() => poll(load));

  const inFlight = $derived(list.items.filter((i) => isInFlight(i) && inArea(i)));

  $effect(() => {
    chatContext.text = JSON.stringify({ area: areas.current, inFlight });
  });
  const oldest = $derived(
    inFlight.reduce<string | null>((min, i) => (min === null || i.openedAt < min ? i.openedAt : min), null),
  );
  const groups = $derived(
    WAITING.map((g) => ({ ...g, items: inFlight.filter((i) => waitingOn(i) === g.id) })).filter(
      (g) => g.items.length > 0,
    ),
  );
</script>

<div class="column">
  <h1 class="page-title">{t("inflight.title", { n: inFlight.length })}</h1>
  <p class="hint intro">
    {oldest ? t("inflight.oldest", { age: age(oldest, list.now) }) : t("inflight.empty")}
  </p>
  <LoadError error={list.error} />

  {#each groups as group (group.id)}
    <section aria-label={t(group.label)}>
      <h2 class="section-label">{t(group.label)}</h2>
      <ul>
        {#each group.items as item (item.id)}
          <li class="row">
            <AreaDot area={item.area} />
            <span class="body">
              <button type="button" class="open" onclick={() => openItem(item.id)}><span class="mono id">{item.id}</span>&nbsp; {item.title}</button>
              <span class="hint label">{item.stateLabel}</span>
            </span>
            {#if item.status}<Chip status={item.status} />{/if}
            <span class="age"><Age since={item.openedAt} now={list.now} /></span>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
</div>

<style>
  .intro {
    margin: 6px 0 32px;
  }

  section {
    padding-bottom: 32px;
  }

  h2 {
    margin-bottom: 8px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 14px 0;
  }

  .body {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .open {
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
  }

  .id {
    font-size: 12px;
    color: var(--faint);
  }

  .label {
    font-size: 13px;
  }

  .age {
    width: 32px;
    text-align: right;
  }
</style>
