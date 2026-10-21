<script lang="ts">
  import { age, isStale } from "../lib/time";

  interface Props {
    since: string;
    now: Date;
  }

  let { since, now }: Props = $props();
</script>

<span class="age mono" class:stale={isStale(since, now)}>{age(since, now)}</span>

<style>
  .age {
    font-size: 13px;
    color: var(--muted);
    white-space: nowrap;
  }

  .stale {
    color: var(--stale);
  }
</style>
