<script lang="ts">
  import { countdownStore } from "../../features/countdown/state/countdown.store";
  import {
    groupStore,
    addGroup,
  } from "../../features/overlay/state/group.store";
  import { formatClock } from "../../shared/time/duration";
  import EyePip from "./EyePip.svelte";
  import OwlMark from "./OwlMark.svelte";
  import type { Subject } from "./types";

  let {
    subject,
    onPickCountdown,
    onPickGroup,
    onNew,
  }: {
    subject: Subject;
    onPickCountdown: (id: number) => void;
    onPickGroup: (id: number) => void;
    onNew: () => void;
  } = $props();

  let newGroupName = $state("");

  const total = $derived($countdownStore.items.length);
  const live = $derived(
    $countdownStore.items.filter((i) => i.state === "Running").length,
  );

  function handleAddGroup() {
    const name = newGroupName.trim();
    if (!name) return;
    void addGroup(name);
    newGroupName = "";
  }
</script>

<aside class="rail">
  <div class="railhead">
    Roost <span class="count">· {total}</span>
  </div>

  {#each $countdownStore.items as item (item.id)}
    <button
      type="button"
      class="strip"
      class:sel={subject === "countdown" &&
        $countdownStore.selectedId === item.id}
      class:sleep={item.state === "Idle"}
      onclick={() => onPickCountdown(item.id)}
    >
      <EyePip state={item.state} />
      <span class="name">{item.label}</span>
      <span class="mini" class:run={item.state === "Running"}
        >{formatClock(item.duration)}</span
      >
    </button>
  {/each}

  <button type="button" class="add" onclick={onNew}>＋ New countdown</button>

  <div class="raildiv"></div>

  <div class="railhead">
    Groups <span class="count">· {$groupStore.items.length}</span>
  </div>

  {#each $groupStore.items as g (g.id)}
    <button
      type="button"
      class="strip group"
      class:sel={subject === "group" && $groupStore.selectedId === g.id}
      onclick={() => onPickGroup(g.id)}
    >
      <span class="gdot"></span>
      <span class="name">{g.name}</span>
      <span class="mini">{g.members.length}</span>
    </button>
  {/each}

  <form
    class="newgroup"
    onsubmit={(e) => {
      e.preventDefault();
      handleAddGroup();
    }}
  >
    <input
      bind:value={newGroupName}
      placeholder="New group…"
      aria-label="New group name"
    />
    <button type="submit" aria-label="Add group">＋</button>
  </form>

  <div class="perch">
    <OwlMark size={30} awake={live > 0} />
    <span class="msg">
      <b>{total} {total === 1 ? "widget" : "widgets"} roosting.</b><br />
      {live > 0 ? `${live} awake and on stream.` : "All quiet on the perch."}
    </span>
  </div>
</aside>

<style>
  .rail {
    grid-area: rail;
    border-right: 1px solid var(--haze-soft);
    display: flex;
    flex-direction: column;
    padding: 18px 14px 0;
    overflow-y: auto;
  }
  .railhead {
    font-family: var(--font-display);
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--dimmer);
    margin: 6px 8px 10px;
  }
  .railhead .count {
    color: var(--dim);
    font-family: var(--font-mono);
    font-weight: 500;
    letter-spacing: 0;
  }

  .strip {
    display: grid;
    grid-template-columns: 18px 1fr auto;
    align-items: center;
    gap: 11px;
    width: 100%;
    text-align: left;
    padding: 11px 12px;
    border-radius: var(--r-input);
    border: 1px solid transparent;
    background: transparent;
    color: inherit;
  }
  .strip:hover {
    background: var(--haze-soft);
  }
  .strip.sel {
    background: var(--ink-card);
    border-color: var(--haze);
  }
  .strip .name {
    font-weight: 600;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .strip.sleep .name {
    color: var(--dim);
    font-weight: 500;
  }
  .strip .mini {
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--dim);
    font-variant-numeric: tabular-nums;
  }
  .strip .mini.run {
    color: var(--eye-bright);
  }

  .gdot {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    justify-self: center;
    background: var(--talon);
  }

  .add {
    margin: 6px 4px 4px;
    padding: 10px 12px;
    border-radius: var(--r-input);
    border: 1px dashed var(--haze);
    background: transparent;
    color: var(--dim);
    font-family: var(--font-body);
    font-size: 13.5px;
    font-weight: 600;
    text-align: left;
  }
  .add:hover {
    color: var(--moon);
    border-color: var(--dim);
  }

  .raildiv {
    height: 1px;
    background: var(--haze-soft);
    margin: 16px 6px;
  }

  .newgroup {
    display: flex;
    gap: 8px;
    margin: 6px 4px 4px;
  }
  .newgroup input {
    flex: 1;
    min-width: 0;
    background: var(--ink);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 9px 11px;
    font-size: 13px;
    color: var(--moon);
  }
  .newgroup input::placeholder {
    color: var(--dimmer);
  }
  .newgroup button {
    flex: none;
    width: 38px;
    border-radius: var(--r-input);
    border: 1px solid var(--haze-soft);
    background: transparent;
    color: var(--dim);
    font-size: 16px;
  }
  .newgroup button:hover {
    color: var(--eye);
    border-color: var(--haze);
  }

  .perch {
    margin-top: auto;
    padding: 16px 8px 18px;
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--dimmer);
  }
  .perch .msg {
    font-size: 12.5px;
    line-height: 1.4;
  }
  .perch .msg b {
    color: var(--dim);
    font-weight: 600;
  }
</style>
