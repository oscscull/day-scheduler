<script lang="ts">
  import { t } from "../../lib/copy";
  import { age } from "../../lib/time";
  import type { Run } from "../../lib/types";

  interface Props {
    runs: Run[];
    now: Date;
  }

  let { runs, now }: Props = $props();
</script>

<section aria-label={t("agent.live")}>
  <h1 class="page-title">
    {runs.length > 0 ? t("agent.working", { n: runs.length }) : t("agent.idle")}
  </h1>
  <p class="hint intro">{t("agent.live")}</p>
  <ul>
    {#each runs as run (run.id)}
      <li class="row">
        <span class="dot"></span>
        <span class="body">
          <span>{run.what}</span>
          <span class="hint meta">{run.target ? t("agent.target", { target: run.target, node: run.node }) : run.node}</span>
        </span>
        <span class="mono elapsed">{age(run.startedAt, now)}</span>
      </li>
    {/each}
  </ul>
</section>

<style>
  .intro {
    margin: 6px 0 20px;
  }

  li {
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
    background: var(--live);
    box-shadow: 0 0 0 4px var(--live-halo);
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

  .elapsed {
    font-size: 13px;
    color: var(--muted);
  }
</style>
