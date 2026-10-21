<script lang="ts">
  import { errorText } from "../lib/api";
  import { t } from "../lib/copy";
  import { changed } from "../lib/refresh.svelte";
  import { route } from "../lib/route";
  import type { Ask } from "../lib/types";

  interface Props {
    ask: Ask;
    node: string | undefined;
    /** Offers dropping the question without answering it. */
    dismissable?: boolean;
  }

  let { ask, node, dismissable = false }: Props = $props();

  /** Long enough to read the notice before the stale question leaves the list. */
  const GONE_NOTICE_MS = 2500;

  let text = $state("");
  let error = $state("");
  let gone = $state(false);
  let sending = $state(false);

  async function send(work: () => Promise<unknown>): Promise<void> {
    sending = true;
    try {
      const outcome = await work();
      error = "";
      text = "";
      if (outcome === "gone") {
        // The screen was stale: say so, then refresh it.
        gone = true;
        setTimeout(changed, GONE_NOTICE_MS);
        return;
      }
      changed();
    } catch (e) {
      error = t("ask.failed", { error: errorText(e) });
    } finally {
      sending = false;
    }
  }

  const answer = (value: string): Promise<void> => send(() => route.answerAsk(node, ask.id, value));
</script>

<div class="answer">
  {#if ask.options.length > 0}
    <div class="options">
      {#each ask.options as option (option)}
        <button type="button" class="button-secondary option" disabled={sending || gone} onclick={() => answer(option)}>{option}</button>
      {/each}
    </div>
  {/if}
  <form onsubmit={(e) => { e.preventDefault(); if (text.trim()) void answer(text.trim()); }}>
    <input
      class="input"
      bind:value={text}
      placeholder={ask.options.length > 0 ? t("ask.writeOther") : t("ask.write")}
      aria-label={ask.options.length > 0 ? t("ask.writeOther") : t("ask.write")}
    />
    {#if text.trim()}
      <button type="submit" class="button-primary" disabled={sending || gone}>{t("ask.send")}</button>
    {/if}
  </form>
  {#if dismissable}
    <button type="button" class="dismiss" disabled={sending || gone} onclick={() => send(() => route.dismissAsk(node, ask.id))}>{t("ask.dismiss")}</button>
  {/if}
  {#if gone}<p class="notice">{t("ask.gone")}</p>{/if}
  {#if error}<p class="error">{error}</p>{/if}
</div>

<style>
  .answer {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .option {
    padding: 6px 14px;
    font-size: 14px;
  }

  form {
    display: flex;
    gap: 8px;
  }

  form .input {
    flex: 1 1 0;
    min-width: 0;
    font-size: 14px;
  }

  .dismiss {
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
  }

  .notice {
    color: var(--muted);
    font-size: 13px;
  }

  .error {
    color: var(--urgent-fg);
    font-size: 13px;
  }
</style>
