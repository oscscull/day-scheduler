import type { ItemInput, WorkItem } from "./types";

/** An item being edited, and the machine it lives on (unset: this one). */
export type Editing = ItemInput & { node?: string | undefined };

/** Which item's detail or editor is open, from any view. */
export const detail = $state({
  id: null as string | null,
  node: undefined as string | undefined,
  editing: null as Editing | null,
});

export const openItem = (id: string, node?: string): void => {
  detail.id = id;
  detail.node = node;
};

export function editItem(item: WorkItem): void {
  detail.editing = {
    id: item.id,
    title: item.title,
    stateLabel: item.stateLabel,
    stage: item.stage,
    status: item.status,
    area: item.area,
    link: item.link,
    node: item.node,
  };
}

export function newItem(area: string): void {
  detail.editing = {
    id: "",
    title: "",
    stateLabel: "",
    stage: "up_next",
    status: null,
    area,
    link: null,
  };
}
