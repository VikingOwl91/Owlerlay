<script lang="ts">
  import { onMount } from "svelte";
  import {
    addGroup,
    groupStore,
    loadGroups,
    removeGroup,
    saveGroup,
    selectGroup,
  } from "../state/group.store";
  import type { Layout } from "../model/group.types";
  import { OVERLAY_SERVER_ORIGIN } from "../../../shared/overlay/origin";

  type CountdownOption = { id: number; label: string };

  let { countdowns }: { countdowns: CountdownOption[] } = $props();

  type Draft = {
    name: string;
    layout: Layout;
    hideIdle: boolean;
    members: number[];
  };

  let newName = $state("");
  let draft = $state<Draft | null>(null);

  // Reset the edit buffer whenever the selection (or the group list) changes.
  $effect(() => {
    const selected =
      $groupStore.items.find((g) => g.id === $groupStore.selectedId) ?? null;
    draft = selected
      ? {
          name: selected.name,
          layout: selected.layout,
          hideIdle: selected.hide_idle,
          members: [...selected.members],
        }
      : null;
  });

  function handleCreate() {
    const name = newName.trim();
    if (!name) return;
    void addGroup(name);
    newName = "";
  }

  function toggleMember(id: number, checked: boolean) {
    if (!draft) return;
    draft.members = checked
      ? [...draft.members, id]
      : draft.members.filter((m) => m !== id);
  }

  function save(id: number) {
    if (!draft) return;
    void saveGroup({
      id,
      name: draft.name,
      layout: draft.layout,
      hide_idle: draft.hideIdle,
      members: draft.members,
    });
  }

  function overlayUrl(id: number): string {
    return `${OVERLAY_SERVER_ORIGIN}/overlay?group=${id}`;
  }

  async function copyUrl(id: number) {
    await navigator.clipboard.writeText(overlayUrl(id));
  }

  onMount(() => {
    void loadGroups();
  });
</script>

<article>
  <header>Overlay Groups</header>

  {#if $groupStore.error}<p class="countdown-error">{$groupStore.error}</p>{/if}

  <!-- svelte-ignore a11y_no_redundant_roles -->
  <fieldset role="group">
    <input bind:value={newName} placeholder="New group name" />
    <button type="button" onclick={handleCreate}>Add group</button>
  </fieldset>

  {#if $groupStore.items.length === 0}
    <p>
      <small>No groups yet. Create one to expose an OBS overlay URL.</small>
    </p>
  {:else}
    <div class="group-list">
      {#each $groupStore.items as g (g.id)}
        <button
          type="button"
          class="secondary {g.id === $groupStore.selectedId ? '' : 'outline'}"
          onclick={() => selectGroup(g.id)}>{g.name}</button
        >
      {/each}
    </div>
  {/if}

  {#if draft}
    <hr />
    <label>
      Name
      <input bind:value={draft.name} />
    </label>
    <label>
      Layout
      <select bind:value={draft.layout}>
        <option value="column">Column</option>
        <option value="row">Row</option>
      </select>
    </label>
    <label>
      <input type="checkbox" bind:checked={draft.hideIdle} />
      Hide idle countdowns
    </label>

    <p><small>Countdowns in this group</small></p>
    {#if countdowns.length === 0}
      <p><small>Create a countdown first.</small></p>
    {:else}
      {#each countdowns as c (c.id)}
        <label>
          <input
            type="checkbox"
            checked={draft.members.includes(c.id)}
            onchange={(e) => toggleMember(c.id, e.currentTarget.checked)}
          />
          {c.label}
        </label>
      {/each}
    {/if}

    <div class="group-actions">
      <button type="button" onclick={() => save($groupStore.selectedId!)}
        >Save</button
      >
      <button
        type="button"
        class="secondary contrast"
        onclick={() => removeGroup($groupStore.selectedId!)}
        >Delete group</button
      >
    </div>

    <div class="source-url">
      <input readonly value={overlayUrl($groupStore.selectedId!)} />
      <button
        type="button"
        class="secondary"
        onclick={() => copyUrl($groupStore.selectedId!)}>Copy</button
      >
    </div>
  {/if}
</article>
