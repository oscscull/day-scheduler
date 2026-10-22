<script lang="ts">
  import { t } from "../../lib/copy";
  import { dayClock } from "../../lib/time";
  import type { AgentTurn } from "../../lib/types";

  interface Props {
    /** The agent's turns with no run here yet: in a session elsewhere, or starting. */
    turns: AgentTurn[];
    now: Date;
  }

  let { turns, now }: Props = $props();
</script>

{#each turns as a (`${a.node ?? ""}|${a.itemId}`)}
  <li class="row">
    <span class="dot"></span>
    <span class="body">
      <span>{a.itemId} · {a.title}</span>
      <span class="hint meta">{a.doing === "session" ? t("agent.session") : t("agent.starting")}</span>
    </span>
    {#if a.until}<span class="mono elapsed">{t("agent.until", { at: dayClock(a.until, now) })}</span>{/if}
  </li>
{/each}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 0;
  }

  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--muted);
  }

  .body {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .meta {
    font-size: 13px;
  }
</style>
