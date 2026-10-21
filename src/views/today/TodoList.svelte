<script lang="ts">
  import AreaDot from "../../components/AreaDot.svelte";
  import { errorText } from "../../lib/api";
  import { ownerOf } from "../../lib/peers.svelte";
  import { route } from "../../lib/route";
  import { ALL, areas } from "../../lib/areas.svelte";
  import { t } from "../../lib/copy";
  import { dueText, isOverdue } from "../../lib/time";
  import type { Todo } from "../../lib/types";

  interface Props {
    todos: Todo[];
    now: Date;
    onChanged: () => void;
  }

  let { todos, now, onChanged }: Props = $props();

  let adding = $state(false);
  let title = $state("");
  let area = $state("");
  let due = $state("");
  let error = $state("");

  function startAdding(): void {
    area = areas.current !== ALL ? areas.current : (areas.list[0]?.id ?? "");
    adding = true;
  }

  async function add(): Promise<void> {
    if (!title.trim()) return;
    try {
      await route.addTodo(ownerOf(area), title.trim(), area, due || null);
      title = "";
      due = "";
      adding = false;
      error = "";
      onChanged();
    } catch (e) {
      error = t("todo.failed", { error: errorText(e) });
    }
  }

  async function toggle(todo: Todo): Promise<void> {
    try {
      await route.setTodoDone(todo.node, todo.id, todo.doneAt === null);
      onChanged();
    } catch (e) {
      error = t("todo.failed", { error: errorText(e) });
    }
  }

  const late = (todo: Todo): boolean => todo.doneAt === null && todo.due !== null && isOverdue(todo.due, now);
</script>

<section aria-label={t("todo.title")}>
  <div class="head">
    <h2>{t("todo.title")}</h2>
    {#if !adding}<button type="button" class="link" onclick={startAdding}>{t("todo.add")}</button>{/if}
  </div>
  <p class="hint sub">{t("todo.hint")}</p>

  {#if adding}
    <form onsubmit={(e) => { e.preventDefault(); void add(); }}>
      <label class="field what">
        {t("todo.what")}
        <input class="input" bind:value={title} placeholder={t("todo.placeholder")} />
      </label>
      <label class="field">
        {t("todo.area")}
        <select class="input" bind:value={area}>
          {#each areas.list as a (a.id)}<option value={a.id}>{a.name}</option>{/each}
        </select>
      </label>
      <label class="field">
        {t("todo.due")}
        <input class="input" type="date" bind:value={due} />
      </label>
      <div class="buttons">
        <button type="button" class="button-secondary" onclick={() => (adding = false)}>{t("todo.cancel")}</button>
        <button type="submit" class="button-primary" disabled={!title.trim()}>{t("todo.save")}</button>
      </div>
    </form>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}

  <ul>
    {#each todos as todo (`${todo.node ?? ""}${todo.id}`)}
      <li class="row" class:done={todo.doneAt !== null}>
        <label class="check">
          <input type="checkbox" checked={todo.doneAt !== null} onchange={() => toggle(todo)} />
          <span class="title">{todo.title}</span>
        </label>
        <AreaDot area={todo.area} />
        <span class="due" class:late={late(todo)}>{todo.due ? dueText(todo.due, now) : ""}</span>
      </li>
    {:else}
      <li class="row hint empty">{t("todo.empty")}</li>
    {/each}
  </ul>
</section>

<style>
  section {
    padding-bottom: 44px;
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .sub {
    margin: 4px 0 12px;
    font-size: 13px;
  }

  .link {
    padding: 8px 0;
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 13px;
  }

  form {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px 12px;
    margin-bottom: 16px;
    padding: 14px;
    border-radius: var(--radius);
    background: var(--surface);
    border: 1px solid var(--line);
  }

  .what {
    grid-column: 1 / -1;
  }

  .buttons {
    grid-column: 1 / -1;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
  }

  .check {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 32px;
    cursor: pointer;
  }

  .check input {
    width: 18px;
    height: 18px;
    flex: none;
    accent-color: var(--ink);
  }

  .done {
    opacity: 0.5;
  }

  .done .title {
    text-decoration: line-through;
  }

  .due {
    width: 84px;
    text-align: right;
    font-size: 13px;
    color: var(--muted);
  }

  .late {
    color: var(--urgent-fg);
  }

  .empty {
    font-size: 14px;
  }
</style>
