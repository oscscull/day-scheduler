<script lang="ts">
  import { api, errorText } from "../../lib/api";
  import { t } from "../../lib/copy";
  import { changed } from "../../lib/refresh.svelte";
  import type { Ask } from "../../lib/types";

  interface Props {
    itemId: string;
    asks: Ask[];
  }

  let { itemId, asks }: Props = $props();

  let question = $state("");
  let context = $state("");
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

  async function add(): Promise<void> {
    if (!question.trim()) return;
    await run(() => api.addAsk(itemId, question.trim(), context.trim()));
    question = "";
    context = "";
  }
</script>

<section aria-label={t("item.asks")}>
  <h3 class="section-label">{t("item.asks")}</h3>
  <ul>
    {#each asks as ask (ask.id)}
      <li class="row">
        <span class="body">
          <span>{ask.question}</span>
          {#if ask.context}<span class="hint small">{ask.context}</span>{/if}
        </span>
        <button type="button" class="button-secondary small" onclick={() => run(() => api.resolveAsk(ask.id))}>
          {t("item.answered")}
        </button>
      </li>
    {:else}
      <li class="row hint small empty">{t("item.noAsks")}</li>
    {/each}
  </ul>
  <form onsubmit={(e) => { e.preventDefault(); void add(); }}>
    <label class="sr-only" for="ask-question">{t("item.addAsk")}</label>
    <input id="ask-question" class="input" bind:value={question} placeholder={t("item.askPlaceholder")} />
    <input class="input" bind:value={context} placeholder={t("item.askContextPlaceholder")} aria-label={t("item.askContextPlaceholder")} />
    <button type="submit" class="button-secondary" disabled={!question.trim()}>{t("item.addAsk")}</button>
  </form>
  {#if error}<p class="error">{error}</p>{/if}
</section>

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

  .empty {
    padding: 10px 0;
  }

  button.small {
    padding: 6px 10px;
    font-size: 13px;
  }

  form {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 8px;
  }

  form .input {
    flex: 1 1 200px;
    font-size: 14px;
  }
</style>
