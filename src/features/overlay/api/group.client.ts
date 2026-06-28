import { invokeCommand } from "../../../shared/tauri/invoke";
import type { GroupDto, Layout } from "../model/group.types";

export async function createGroup(name: string): Promise<number> {
  return invokeCommand<number>("group_create", { name });
}

export async function listGroups(): Promise<GroupDto[]> {
  return invokeCommand<GroupDto[]>("group_list", {});
}

export async function updateGroup(
  id: number,
  name: string,
  members: number[],
  layout: Layout,
  hideIdle: boolean,
): Promise<void> {
  await invokeCommand<void>("group_update", {
    id,
    name,
    members,
    layout,
    hideIdle,
  });
}

export async function deleteGroup(id: number): Promise<void> {
  await invokeCommand<void>("group_delete", { id });
}
