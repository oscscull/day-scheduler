<script lang="ts">
  import { api, errorText } from "../lib/api";
  import { chatContext } from "../lib/chat.svelte";
  import { t } from "../lib/copy";
  import { changed } from "../lib/refresh.svelte";

  interface Props {
    page: string;
    suggestions: string[];
  }

  interface Message {
    from: "you" | "claude";
    text: string;
  }

  let { page, suggestions }: Props = $props();

  // Lives only while the bubble is mounted: nothing here is stored.
  let open = $state(false);
  let draft = $state("");
  let busy = $state(false);
  let error = $state("");
  let messages = $state<Message[]>([]);

  async function send(): Promise<void> {
    const text = draft.trim();
    if (!text || busy) return;
    messages.push({ from: "you", text });
    draft = "";
    busy = true;
    error = "";
    try {
      messages.push({ from: "claude", text: await api.askClaude(page, chatContext.text, text) });
      // Claude may have changed things through the Quarterdeck tools.
      changed();
    } catch (e) {
      error = t("chat.failed", { error: errorText(e) });
    } finally {
      busy = false;
    }
  }
</script>

<div class="dock">
  {#if open}
    <section aria-label={t("chat.title")}>
      <div class="head">
        <h2>{t("chat.title")}</h2>
        <button type="button" class="close" aria-label={t("chat.close")} onclick={() => (open = false)}>×</button>
      </div>
      <p class="hint context">{t("chat.looking", { page })}</p>
      {#if messages.length === 0}
        <div class="suggestions">
          {#each suggestions as text (text)}
            <button type="button" onclick={() => (draft = text)}>{text}</button>
          {/each}
        </div>
      {:else}
        <ol class="messages">
          {#each messages as message, i (i)}
            <li class={message.from}>{message.text}</li>
          {/each}
        </ol>
      {/if}
      {#if busy}<p class="hint context">{t("chat.thinking")}</p>{/if}
      {#if error}<p class="error context">{error}</p>{/if}
      <form onsubmit={(e) => { e.preventDefault(); void send(); }}>
        <label class="grow">
          <span class="sr-only">{t("chat.placeholder")}</span>
          <input bind:value={draft} placeholder={t("chat.placeholder")} />
        </label>
        <button type="submit" class="send" aria-label={t("chat.send")} disabled={busy}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 19V5M5 12l7-7 7 7"></path></svg>
        </button>
      </form>
    </section>
  {/if}
  <button type="button" class="bubble" aria-label={t("chat.open")} onclick={() => (open = !open)}>
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 12a8 8 0 0 1-11.6 7.1L4 20l1-4.6A8 8 0 1 1 21 12z"></path></svg>
  </button>
</div>

<style>
  .dock {
    position: fixed;
    right: 24px;
    bottom: 24px;
    z-index: 50;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 12px;
    font-size: 14px;
  }

  section {
    width: 360px;
    max-width: calc(100vw - 48px);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 16px;
    box-shadow: 0 12px 40px rgba(25, 25, 23, 0.12);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px 10px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .close {
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    font-size: 18px;
  }

  .context {
    margin: 0 16px 12px;
    font-size: 13px;
  }

  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 16px 14px;
  }

  .suggestions button {
    text-align: left;
    padding: 9px 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface-soft);
    font-size: 13px;
  }

  .messages {
    max-height: 320px;
    overflow-y: auto;
    padding: 0 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .messages li {
    padding: 8px 12px;
    border-radius: 12px;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  .you {
    align-self: flex-end;
    background: var(--ink);
    color: #fff;
  }

  .claude {
    align-self: flex-start;
    background: var(--surface-soft);
  }

  form {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 12px 12px 12px 16px;
    border-top: 1px solid var(--line-soft);
  }

  .grow {
    flex: 1 1 0;
    min-width: 0;
  }

  input {
    width: 100%;
    padding: 10px 0;
    border: 0;
    outline: none;
    background: transparent;
  }

  .send,
  .bubble {
    border: 0;
    border-radius: 50%;
    background: var(--ink);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .send {
    width: 36px;
    height: 36px;
    flex: none;
  }

  .bubble {
    width: 52px;
    height: 52px;
    box-shadow: 0 6px 20px rgba(25, 25, 23, 0.16);
  }
</style>
