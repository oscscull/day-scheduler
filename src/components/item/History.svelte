<script lang="ts">
  import { t } from "../../lib/copy";
  import { dayClock } from "../../lib/time";
  import { passText } from "../../lib/turn";
  import type { Change, ItemEvent } from "../../lib/types";

  interface Props {
    events: ItemEvent[];
    now: Date;
  }

  let { events, now }: Props = $props();

  function describe(change: Change): string {
    switch (change.kind) {
      case "created":
        return t("history.created");
      case "stage":
        return t("history.stage", { stage: t(`stage.${change.stage}`) });
      case "status":
        return change.status
          ? t("history.status", { status: t(`status.${change.status}`) })
          : t("history.statusCleared");
      case "label":
        return t("history.label", { label: change.label });
      case "note":
        return t("history.note", { text: change.text });
      case "turn":
        return passText(change.turn, change.why);
    }
  }
</script>

{#if events.length > 0}
  <section aria-label={t("item.history")}>
    <h3 class="section-label">{t("item.history")}</h3>
    <ol>
      {#each events as e (e.id)}
        <li>
          <span class="mono when">{dayClock(e.at, now)}</span>
          <span>{describe(e.change)}</span>
        </li>
      {/each}
    </ol>
  </section>
{/if}

<style>
  h3 {
    margin-bottom: 6px;
  }

  ol {
    max-height: 200px;
    overflow-y: auto;
  }

  li {
    display: flex;
    gap: 12px;
    padding: 6px 0;
    font-size: 14px;
    color: #4a4843;
  }

  .when {
    width: 84px;
    flex: none;
    font-size: 12px;
    color: var(--muted);
  }
</style>
