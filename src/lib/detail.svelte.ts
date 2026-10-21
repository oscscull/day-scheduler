import type { ItemInput, WorkItem } from "./types";

/** Which item's detail or editor is open, from any view. */
export const detail = $state({
  id: null as string | null,
  editing: null as ItemInput | null,
});

export const openItem = (id: string): void => {
  detail.id = id;
};

export function editItem(item: WorkItem): void {
  detail.editing = {
    id: item.id,
    title: item.title,
    stateLabel: item.stateLabel,
    stage: item.stage,
    status: item.status,
    holder: item.holder,
    area: item.area,
    link: item.link,
  };
}

export function newItem(area: string): void {
  detail.editing = {
    id: "",
    title: "",
    stateLabel: "",
    stage: "up_next",
    status: null,
    holder: "",
    area,
    link: null,
  };
}
