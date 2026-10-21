<script lang="ts">
  import { t } from "../lib/copy";
  import type { Status } from "../lib/types";

  interface Props {
    status: Status;
    /** Optional detail after the label, e.g. "Review · Sam". */
    detail?: string | undefined;
  }

  let { status, detail }: Props = $props();
</script>

<span class="chip {status}">
  {t(`status.${status}`)}{#if detail}&nbsp;· {detail}{/if}
</span>

<style>
  .chip {
    flex: none;
    padding: 3px 9px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
  }

  .decision { background: var(--chip-decision-bg); color: var(--chip-decision-fg); }
  .review { background: var(--chip-review-bg); color: var(--chip-review-fg); }
  .ci { background: var(--chip-ci-bg); color: var(--chip-ci-fg); }
  .ci_failed { background: var(--urgent-bg); color: var(--urgent-fg); }
  .dependency { background: var(--chip-dependency-bg); color: var(--chip-dependency-fg); }
  .approved { background: var(--chip-approved-bg); color: var(--chip-approved-fg); }
  .proceeding { background: var(--chip-proceeding-bg); color: var(--chip-proceeding-fg); }
  .queued { background: var(--chip-queued-bg); color: var(--chip-queued-fg); }
</style>
