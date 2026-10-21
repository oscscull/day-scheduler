<script lang="ts">
  import { errorText } from "../../lib/api";
  import { route } from "../../lib/route";
  import { t } from "../../lib/copy";
  import { changed } from "../../lib/refresh.svelte";
  import type { Ask } from "../../lib/types";

  interface Props {
    node: string | undefined;
    asks: Ask[];
  }

  let { node, asks }: Props = $props();

  let error = $state("");

  async function run(work: () => Promise<unknown>): Promise<void> {
    try {
      await work();
      error = "";
      changed();
    } catch (e) {
      error = t("item.failed", { error: errorText(e) });
    }
  }

</script>

{#if asks.length > 0}
<section aria-label={t("item.asks")}>
  <h3 class="section-label">{t("item.asks")}</h3>
  <ul>
    {#each asks as ask (ask.id)}
      <li class="row">
        <span class="body">
          <span>{ask.question}</span>
          {#if ask.context}<span class="hint small">{ask.context}</span>{/if}
        </span>
        <button type="button" class="button-secondary small" onclick={() => run(() => route.resolveAsk(node, ask.id))}>
          {t("item.answered")}
        </button>
      </li>
    {/each}
  </ul>
  {#if error}<p class="error">{error}</p>{/if}
</section>
{/if}

<style>
  h3 {
    margin-bottom: 6px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
  }

  .body {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .small {
    font-size: 13px;
  }


  button.small {
    padding: 6px 10px;
    font-size: 13px;
  }

</style>
