<script lang="ts">
  import { api } from "../../lib/api";
  import { t } from "../../lib/copy";
  import { dayClock } from "../../lib/time";
  import type { Explanation } from "../../lib/types";

  interface Props {
    expr: string;
    now: Date;
  }

  let { expr, now }: Props = $props();

  const DEBOUNCE_MS = 250;
  let explanation = $state<Explanation | null>(null);
  let invalid = $state(false);

  // Re-explain shortly after the user stops typing.
  $effect(() => {
    const current = expr;
    const timer = setTimeout(() => {
      api
        .explainCron(current)
        .then((e) => { explanation = e; invalid = false; })
        .catch(() => { explanation = null; invalid = true; });
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });
</script>

<div class="box">
  {#if invalid}
    <span class="error">{t("schedule.cronInvalid")}</span>
  {:else if explanation}
    <span class="when">{explanation.when}</span>
    <div class="fields">
      {#each explanation.fields as f (f.unit)}
        <span class="field-part">
          <span class="mono value">{f.value}</span>
          <span class="means">{t(`unit.${f.unit}`)}: {f.means}</span>
        </span>
      {/each}
    </div>
    <span class="hint next">
      {t("schedule.next", { times: explanation.nextRuns.map((r) => dayClock(r, now)).join(", ") })}
    </span>
  {/if}
</div>

<style>
  .box {
    padding: 12px 14px;
    border-radius: var(--radius);
    background: var(--surface-soft);
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 44px;
  }

  .when {
    font-size: 15px;
  }

  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
  }

  .field-part {
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-size: 12px;
  }

  .means {
    color: var(--muted);
  }

  .next {
    font-size: 13px;
  }
</style>
