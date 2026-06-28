import { writable } from "svelte/store";
import type { GroupDto } from "../model/group.types";
import {
  createGroup,
  deleteGroup,
  listGroups,
  updateGroup,
} from "../api/group.client";

type GroupStateStore = {
  items: GroupDto[];
  selectedId: number | null;
  loading: boolean;
  error: string | null;
};

const initialStateStore: GroupStateStore = {
  items: [],
  selectedId: null,
  loading: false,
  error: null,
};

const { subscribe, update } = writable(initialStateStore);

function toMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function loadGroups() {
  update((s) => ({ ...s, loading: true, error: null }));
  try {
    const items = await listGroups();
    update((s) => ({ ...s, items }));
  } catch (error) {
    update((s) => ({ ...s, error: toMessage(error) }));
  } finally {
    update((s) => ({ ...s, loading: false }));
  }
}

export async function addGroup(name: string) {
  update((s) => ({ ...s, loading: true, error: null }));
  try {
    const id = await createGroup(name);
    const items = await listGroups();
    update((s) => ({ ...s, items, selectedId: id }));
  } catch (error) {
    update((s) => ({ ...s, error: toMessage(error) }));
  } finally {
    update((s) => ({ ...s, loading: false }));
  }
}

export async function removeGroup(id: number) {
  update((s) => ({ ...s, loading: true, error: null }));
  try {
    await deleteGroup(id);
    const items = await listGroups();
    update((s) => ({
      ...s,
      items,
      selectedId: s.selectedId === id ? null : s.selectedId,
    }));
  } catch (error) {
    update((s) => ({ ...s, error: toMessage(error) }));
  } finally {
    update((s) => ({ ...s, loading: false }));
  }
}

export async function saveGroup(group: GroupDto) {
  update((s) => ({ ...s, loading: true, error: null }));
  try {
    await updateGroup(
      group.id,
      group.name,
      group.members,
      group.layout,
      group.hide_idle,
    );
    const items = await listGroups();
    update((s) => ({ ...s, items }));
  } catch (error) {
    update((s) => ({ ...s, error: toMessage(error) }));
  } finally {
    update((s) => ({ ...s, loading: false }));
  }
}

export function selectGroup(id: number | null) {
  update((s) => ({ ...s, selectedId: id }));
}

export const groupStore = { subscribe };
