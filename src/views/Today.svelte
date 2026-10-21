<script lang="ts">
  import LoadError from "../components/LoadError.svelte";
  import { api, errorText } from "../lib/api";
  import { chatContext } from "../lib/chat.svelte";
  import { t } from "../lib/copy";
  import { poll } from "../lib/poll";
  import { clock, dayClock, until } from "../lib/time";
  import type { TodayView } from "../lib/types";

  let data = $state<TodayView | null>(null);
  let error = $state("");
  let now = $state(new Date());

  async function load(): Promise<void> {
    try {
      data = await api.today();
      now = new Date();
      error = "";
      chatContext.text = JSON.stringify(data);
    } catch (e) {
      error = errorText(e);
    }
  }

  $effect(() => poll(load));

  const next = $derived(data?.upcoming[0]);
  const then = $derived(data?.upcoming[1]);
</script>

<div class="column">
  <LoadError {error} />
  {#if data}
    <section class="next" aria-label={t("today.next")}>
      {#if next}
        <span class="section-label">{t("today.next")}</span>
        <h1>{next.title}</h1>
        <p class="when">
          <span class="mono">{clock(next.startsAt)}</span>
          <span class="muted">{t("today.in", { when: until(next.startsAt, now) })}</span>
          {#if next.location}<span class="muted">{t("today.where", { location: next.location })}</span>{/if}
        </p>
        <p class="hint">
          {then ? t("today.then", { title: then.title, when: dayClock(then.startsAt, now) }) : t("today.nothingAfter")}
        </p>
      {:else}
        <p class="hint">{t("today.noCommitments")}</p>
      {/if}
    </section>

    <section aria-label={t("today.priorities")} class="block">
      <h2>{t("today.priorities")}</h2>
      <p class="hint sub">{t("today.prioritiesHint")}</p>
      <ul>
        {#each data.priorities as p (p.ask.id)}
          <li class="row priority">
            <span>{p.ask.question}</span>
            <span class="hint small">{p.ask.context} · {t("today.unblocks", { id: p.itemId })}</span>
          </li>
        {:else}
          <li class="hint">{t("today.noPriorities")}</li>
        {/each}
      </ul>
    </section>

    <section aria-label={t("today.fromClaude")}>
      <h2 class="section-label">{t("today.fromClaude")}</h2>
      <ul class="activity">
        {#each data.activity as a (a.id)}
          <li class:unread={a.unread}>
            <span class="dot"></span>
            <span class="text">{a.text}</span>
            <span class="mono time">{clock(a.at)}</span>
          </li>
        {/each}
      </ul>
      <p class="hint working">{t("today.working", { n: data.running })}</p>
    </section>
  {/if}
</div>

<style>
  .next {
    padding: 24px 0 48px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h1 {
    font-size: 44px;
    font-weight: 600;
    letter-spacing: -0.025em;
    line-height: 1.05;
  }

  .when {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 18px;
  }

  .muted {
    color: var(--muted);
  }

  .block {
    padding-bottom: 48px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .sub {
    margin: 4px 0 16px;
    font-size: 13px;
  }

  .priority {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 14px 0;
  }

  .small {
    font-size: 13px;
  }

  .activity {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .activity li {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 6px 0;
    font-size: 14px;
    color: var(--muted);
  }

  .activity .unread {
    color: var(--ink);
  }

  .dot {
    width: 6px;
    height: 6px;
    flex: none;
    border-radius: 50%;
    transform: translateY(-2px);
  }

  .unread .dot {
    background: var(--accent);
  }

  .text {
    flex: 1 1 0;
    min-width: 0;
  }

  .time {
    font-size: 12px;
    color: var(--faint);
  }

  .working {
    margin: 12px 0 0 16px;
    font-size: 13px;
  }
</style>
