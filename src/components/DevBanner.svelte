<script lang="ts">
  import { t } from "../lib/copy";

  // `tauri dev` serves the frontend through Vite's dev server; a release is a built bundle.
  const dev = import.meta.env.DEV;
</script>

{#if dev}
  <div class="banner" role="note">{t("dev.banner")}</div>
{/if}

<style>
  .banner {
    position: sticky;
    top: 0;
    z-index: 70;
    padding: 4px 16px;
    background: var(--chip-decision-bg);
    color: var(--chip-decision-fg);
    font-size: 12px;
    font-weight: 600;
    text-align: center;
  }
</style>
