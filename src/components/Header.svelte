<script lang="ts">
  import { longDate } from "../lib/time";
  import type { ViewId } from "../lib/views";
  import AreaSwitch from "./AreaSwitch.svelte";
  import ViewSwitch from "./ViewSwitch.svelte";

  interface Props {
    view: ViewId;
    onSelect: (view: ViewId) => void;
  }

  let { view, onSelect }: Props = $props();

  // The window stays open for days, so re-read the clock and let the date roll over at midnight.
  const REFRESH_MS = 60_000;
  let today = $state(new Date());

  $effect(() => {
    const timer = setInterval(() => (today = new Date()), REFRESH_MS);
    return () => clearInterval(timer);
  });
</script>

<header>
  <span class="side">{longDate(today)}</span>
  <ViewSwitch {view} {onSelect} />
  <div class="side end"><AreaSwitch /></div>
</header>

<style>
  header {
    max-width: var(--wide);
    margin: 0 auto;
    padding: 20px 24px 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    color: var(--muted);
  }

  .side {
    flex: 1 1 160px;
  }

  .end {
    display: flex;
    justify-content: flex-end;
  }
</style>
