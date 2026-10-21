<script lang="ts">
  import { t } from "../lib/copy";
  import { VIEWS, viewLabel, type ViewId } from "../lib/views";

  interface Props {
    view: ViewId;
    onSelect: (view: ViewId) => void;
  }

  let { view, onSelect }: Props = $props();
</script>

<nav aria-label="Views">
  {#each VIEWS as id (id)}
    <button
      type="button"
      class:active={id === view}
      aria-current={id === view ? "page" : undefined}
      onclick={() => onSelect(id)}
    >
      {t(viewLabel(id))}
    </button>
  {/each}
</nav>

<style>
  nav {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    padding: 3px;
    border-radius: 9px;
    background: var(--switch-track);
  }

  button {
    padding: 8px 16px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--muted);
    font-size: 14px;
  }

  .active {
    background: var(--surface);
    color: var(--ink);
    font-weight: 500;
  }
</style>
