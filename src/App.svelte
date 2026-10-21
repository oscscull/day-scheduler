<script lang="ts">
  import ChatBubble from "./components/ChatBubble.svelte";
  import Header from "./components/Header.svelte";
  import ItemDetailModal from "./components/item/ItemDetailModal.svelte";
  import ItemEditor from "./components/item/ItemEditor.svelte";
  import { detail, openItem } from "./lib/detail.svelte";
  import { loadAreas } from "./lib/areas.svelte";
  import { t } from "./lib/copy";
  import { suggestionKeys, viewLabel, type ViewId } from "./lib/views";
  import Agent from "./views/Agent.svelte";
  import Board from "./views/Board.svelte";
  import InFlight from "./views/InFlight.svelte";
  import Priorities from "./views/Priorities.svelte";
  import Today from "./views/Today.svelte";

  let view = $state<ViewId>("today");

  $effect(() => {
    void loadAreas();
  });
</script>

<Header {view} onSelect={(next) => (view = next)} />

{#if view === "today"}
  <Today />
{:else if view === "priorities"}
  <Priorities />
{:else if view === "board"}
  <Board />
{:else if view === "inflight"}
  <InFlight />
{:else}
  <Agent />
{/if}

{#key view}
  <ChatBubble page={t(viewLabel(view))} suggestions={suggestionKeys(view).map((key) => t(key))} />
{/key}

{#if detail.id}
  <ItemDetailModal id={detail.id} onClose={() => (detail.id = null)} />
{/if}

{#if detail.editing}
  <ItemEditor
    item={detail.editing}
    onClose={() => (detail.editing = null)}
    onSaved={(id) => { detail.editing = null; openItem(id); }}
  />
{/if}
