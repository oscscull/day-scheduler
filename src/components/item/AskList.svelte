<script lang="ts">
  import AnswerBox from "../AnswerBox.svelte";
  import { t } from "../../lib/copy";
  import type { Ask } from "../../lib/types";

  interface Props {
    node: string | undefined;
    asks: Ask[];
  }

  let { node, asks }: Props = $props();
</script>

{#if asks.length > 0}
  <section aria-label={t("item.asks")}>
    <h3 class="section-label">{t("item.asks")}</h3>
    <ul>
      {#each asks as ask (ask.id)}
        <li>
          <p class="question">{ask.question}</p>
          {#if ask.context}<p class="hint small">{ask.context}</p>{/if}
          <AnswerBox {ask} {node} dismissable />
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  h3 {
    margin-bottom: 6px;
  }

  li {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 0 14px;
  }

  li + li {
    border-top: 1px solid var(--line-soft);
  }

  .question {
    font-size: 15px;
  }

  .small {
    font-size: 13px;
    margin-bottom: 6px;
    overflow-wrap: anywhere;
  }
</style>
