<script lang="ts">
  import { t } from "../../lib/copy";

  interface Props {
    count: number;
    onOpen: () => void;
  }

  let { count, onOpen }: Props = $props();
</script>

{#if count > 0}
  <button type="button" onclick={onOpen}>
    <span class="dot"></span>{t("urgent.pill", { n: count })}
  </button>
{/if}

<style>
  button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border: 0;
    border-radius: 999px;
    background: var(--urgent-bg);
    color: var(--urgent-fg);
    font-size: 13px;
    font-weight: 500;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--urgent-dot);
  }
</style>
