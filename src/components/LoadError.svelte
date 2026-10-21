<script lang="ts">
  import { t } from "../lib/copy";

  interface Props {
    error: string;
  }

  let { error }: Props = $props();
</script>

{#if error}
  <p class="error" role="alert">{t("app.loadFailed", { error })}</p>
{/if}
