<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import App from "./App.svelte";
  import AlertWindow from "./views/AlertWindow.svelte";

  // The same frontend serves the main window and the meeting alert window.
  const isAlert = getCurrentWindow().label === "alert";
</script>

{#if isAlert}
  <AlertWindow />
{:else}
  <App />
{/if}
