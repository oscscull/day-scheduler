<script lang="ts">
  import { api } from "../../lib/api";
  import { t } from "../../lib/copy";

  let command = $state("");
  let copied = $state(false);

  $effect(() => {
    api.mcpConnectCommand().then((c) => (command = c)).catch(() => (command = ""));
  });

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(command);
      copied = true;
    } catch {
      copied = false;
    }
  }
</script>

{#if command}
  <section aria-label={t("connect.title")}>
    <h2 class="section-label">{t("connect.title")}</h2>
    <p class="hint">{t("connect.hint")}</p>
    <div class="box">
      <code class="mono">{command}</code>
      <button type="button" class="button-secondary" onclick={copy}>{copied ? t("connect.copied") : t("connect.copy")}</button>
    </div>
  </section>
{/if}

<style>
  section {
    margin-top: 44px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hint {
    font-size: 13px;
  }

  .box {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
  }

  code {
    flex: 1 1 0;
    min-width: 0;
    overflow-x: auto;
    white-space: nowrap;
    font-size: 12px;
  }
</style>
