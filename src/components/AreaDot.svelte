<script lang="ts">
  import { ALL, areas, findArea } from "../lib/areas.svelte";

  interface Props {
    area: string;
  }

  let { area }: Props = $props();

  const found = $derived(findArea(area));
</script>

<!-- Only useful when several areas are on screen together. -->
{#if areas.current === ALL && found}
  <span class="dot" style:background={found.color} title={found.name}></span>
{/if}

<style>
  .dot {
    width: 7px;
    height: 7px;
    flex: none;
    border-radius: 50%;
  }
</style>
