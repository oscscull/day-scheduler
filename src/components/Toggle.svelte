<script lang="ts">
  interface Props {
    on: boolean;
    label: string;
    onChange: (on: boolean) => void;
  }

  let { on, label, onChange }: Props = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={on}
  aria-label={label}
  class:on
  onclick={() => onChange(!on)}
>
  <span></span>
</button>

<style>
  button {
    flex: none;
    width: 44px;
    height: 26px;
    padding: 3px;
    border: 0;
    border-radius: 13px;
    background: var(--toggle-off);
    display: flex;
    justify-content: flex-start;
  }

  .on {
    background: var(--live);
    justify-content: flex-end;
  }

  span {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
  }
</style>
