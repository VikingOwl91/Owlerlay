<script lang="ts">
  import { groupStore, saveGroup, removeGroup } from "../state/group.store";
  import { countdownStore } from "../../countdown/state/countdown.store";
  import type { Layout } from "../model/group.types";
  import { OVERLAY_SERVER_ORIGIN } from "../../../shared/overlay/origin";

  type Draft = {
    name: string;
    layout: Layout;
    hideIdle: boolean;
    members: number[];
  };

  const selected = $derived(
    $groupStore.items.find((g) => g.id === $groupStore.selectedId) ?? null,
  );

  let draft = $state<Draft | null>(null);
  // Reset the edit buffer whenever the selected group changes.
  $effect(() => {
    draft = selected
      ? {
          name: selected.name,
          layout: selected.layout,
          hideIdle: selected.hide_idle,
          members: [...selected.members],
        }
      : null;
  });

  function toggleMember(id: number, checked: boolean) {
    if (!draft) return;
    draft.members = checked
      ? [...draft.members, id]
      : draft.members.filter((m) => m !== id);
  }

  function save() {
    if (!selected || !draft) return;
    void saveGroup({
      id: selected.id,
      name: draft.name,
      layout: draft.layout,
      hide_idle: draft.hideIdle,
      members: draft.members,
    });
  }

  const url = $derived(
    selected ? `${OVERLAY_SERVER_ORIGIN}/overlay?group=${selected.id}` : "",
  );
  function copyUrl() {
    if (url) navigator.clipboard.writeText(url).catch((e) => console.error(e));
  }
</script>

{#if selected && draft}
  <div class="detail">
    <div class="head">
      <h1>{selected.name}</h1>
      <span class="chip">Group</span>
    </div>
    <p class="sub">An OBS browser source bundling these countdowns.</p>

    <section class="card">
      <div class="row">
        <label for="g-name">Name</label>
        <input id="g-name" class="text" bind:value={draft.name} />
      </div>
      <div class="row">
        <label for="g-layout">Layout</label>
        <select id="g-layout" class="text" bind:value={draft.layout}>
          <option value="column">Column</option>
          <option value="row">Row</option>
        </select>
      </div>
      <div class="row">
        <label for="g-hide">Hide idle countdowns</label>
        <input
          id="g-hide"
          type="checkbox"
          class="toggle"
          bind:checked={draft.hideIdle}
        />
      </div>

      <div class="seg">Countdowns in this group</div>
      {#if $countdownStore.items.length === 0}
        <p class="empty">Create a countdown first.</p>
      {:else}
        <div class="members">
          {#each $countdownStore.items as c (c.id)}
            <label class="member">
              <input
                type="checkbox"
                checked={draft.members.includes(c.id)}
                onchange={(e) => toggleMember(c.id, e.currentTarget.checked)}
              />
              {c.label}
            </label>
          {/each}
        </div>
      {/if}
    </section>

    <div class="transport">
      <button class="btn primary" onclick={save}>Save</button>
      <button class="btn danger" onclick={() => removeGroup(selected.id)}
        >Delete group</button
      >
    </div>

    <div class="urlrow">
      <span class="url">{url}</span>
      <button class="btn ghost small" onclick={copyUrl}>Copy</button>
    </div>
  </div>
{/if}

<style>
  .detail {
    max-width: 880px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .head h1 {
    font-size: 26px;
  }
  .chip {
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--talon);
    border: 1px solid var(--haze);
    background: var(--haze-soft);
    padding: 5px 11px;
    border-radius: var(--r-pill);
  }
  .sub {
    color: var(--dim);
    font-size: 13.5px;
    margin: 4px 0 26px;
  }

  .card {
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-card);
    background: var(--ink-raised);
    padding: 20px 22px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px 16px;
    padding: 10px 0;
  }
  .row + .row {
    border-top: 1px solid var(--haze-soft);
  }
  .row label {
    color: var(--dim);
    font-size: 13.5px;
  }
  .text {
    background: var(--ink);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 9px 12px;
    color: var(--moon);
    font-size: 14px;
    flex: 1 1 220px;
    min-width: 0;
    max-width: 340px;
  }
  select.text {
    appearance: none;
    cursor: pointer;
    color: var(--moon);
    background-color: var(--ink);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 12' fill='none' stroke='%239d94ae' stroke-width='1.6' stroke-linecap='round'%3E%3Cpath d='M2.5 4.5 6 8l3.5-3.5'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 12px center;
    background-size: 12px;
    padding-right: 34px;
  }
  select.text option {
    background: var(--ink-raised);
    color: var(--moon);
  }

  .seg {
    color: var(--dimmer);
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    margin: 18px 0 12px;
  }
  .empty {
    color: var(--dim);
    font-size: 13.5px;
  }
  .members {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 22px;
  }
  .member {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: 14px;
    color: var(--moon);
    cursor: pointer;
  }
  .member input {
    accent-color: var(--eye);
    width: 16px;
    height: 16px;
  }

  .toggle {
    appearance: none;
    flex: none;
    width: 40px;
    height: 23px;
    border-radius: 999px;
    background: var(--haze);
    position: relative;
    cursor: pointer;
  }
  .toggle:checked {
    background: var(--eye);
  }
  .toggle::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: #fff;
    transition: left 0.15s;
  }
  .toggle:checked::after {
    left: 20px;
  }

  .transport {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 22px;
  }
  .btn {
    font-family: var(--font-body);
    font-weight: 700;
    font-size: 14.5px;
    white-space: nowrap;
    border-radius: var(--r-pill);
    padding: 13px 24px;
    border: 1px solid transparent;
  }
  .btn.small {
    padding: 9px 18px;
    font-size: 13px;
  }
  .btn.primary {
    background: var(--eye);
    color: #2a1c05;
    box-shadow: var(--glow);
  }
  .btn.primary:hover {
    background: var(--eye-bright);
  }
  .btn.ghost {
    background: transparent;
    border-color: var(--haze);
    color: var(--moon);
  }
  .btn.ghost:hover {
    border-color: var(--dim);
  }
  .btn.danger {
    background: transparent;
    color: var(--dimmer);
  }
  .btn.danger:hover {
    color: var(--talon);
  }

  .urlrow {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 22px;
    background: var(--ink-raised);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 10px 10px 10px 14px;
    max-width: 560px;
  }
  .url {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
