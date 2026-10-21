<script lang="ts">
  import { t, type CopyKey } from "../../lib/copy";
  import type { Autonomy } from "../../lib/types";

  interface Props {
    value: Autonomy;
    onChange: (value: Autonomy) => void;
  }

  let { value, onChange }: Props = $props();

  const CHOICES: { id: Autonomy; label: CopyKey; hint: CopyKey }[] = [
    { id: "ask", label: "autonomy.ask", hint: "autonomy.askHint" },
    { id: "act", label: "autonomy.act", hint: "autonomy.actHint" },
  ];
</script>

<fieldset>
  <legend class="field">{t("schedule.autonomy")}</legend>
  <div class="choices">
    {#each CHOICES as c (c.id)}
      <button type="button" aria-pressed={value === c.id} class:picked={value === c.id} onclick={() => onChange(c.id)}>
        <span class="label">{t(c.label)}</span>
        <span class="hint small">{t(c.hint)}</span>
      </button>
    {/each}
  </div>
</fieldset>

<style>
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    margin-bottom: 8px;
  }

  .choices {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  button {
    flex: 1 1 200px;
    text-align: left;
    padding: 12px 14px;
    border: 1.5px solid var(--line);
    border-radius: var(--radius);
    background: var(--surface);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .picked {
    border-color: var(--ink);
    background: var(--surface-soft);
  }

  .label {
    font-weight: 500;
  }

  .small {
    font-size: 13px;
  }
</style>
