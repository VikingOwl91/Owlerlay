<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { countdownStore } from "../../features/countdown/state/countdown.store";
  import {
    loadGroups,
    selectGroup,
  } from "../../features/overlay/state/group.store";
  import OwlMark from "./OwlMark.svelte";
  import Roost from "./Roost.svelte";
  import Stage from "./Stage.svelte";
  import type { Subject } from "./types";

  let subject = $state<Subject>(null);
  let cleanup: (() => void) | null = null;

  const liveCount = $derived(
    $countdownStore.items.filter((i) => i.state === "Running").length,
  );

  function pickCountdown(id: number) {
    countdownStore.select(id);
    selectGroup(null);
    subject = "countdown";
  }
  function pickGroup(id: number) {
    selectGroup(id);
    subject = "group";
  }
  function startCreate() {
    subject = "create";
  }
  function openSettings() {
    subject = "settings";
  }
  function finishCreate() {
    subject = "countdown"; // the store auto-selects the new countdown
  }

  onMount(async () => {
    await countdownStore.loadList();
    cleanup = await countdownStore.initStoreListeners();
    void loadGroups();
    const s = get(countdownStore);
    if (s.items.length > 0) {
      countdownStore.select(s.items[0].id);
      subject = "countdown";
    }
  });
  onDestroy(() => cleanup?.());
</script>

<div class="app">
  <header class="topbar">
    <div class="brand">
      <OwlMark size={26} />
      <span class="wordmark">Owler<b>lay</b></span>
    </div>
    <span class="tagline">your overlays, after dark</span>
    <div class="spacer"></div>
    {#if liveCount > 0}
      <span class="livepill"><span class="dot"></span>{liveCount} live</span>
    {/if}
    <button
      type="button"
      class="gear"
      class:active={subject === "settings"}
      onclick={openSettings}
      aria-label="Phone remote settings"
      title="Phone remote"
    >
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z"
          stroke="currentColor"
          stroke-width="1.6"
        />
        <path
          d="M19.4 13a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1.08-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1.08 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
          stroke="currentColor"
          stroke-width="1.6"
        />
      </svg>
    </button>
  </header>

  <Roost
    {subject}
    onPickCountdown={pickCountdown}
    onPickGroup={pickGroup}
    onNew={startCreate}
  />

  <Stage {subject} onCreated={finishCreate} onStartCreate={startCreate} />
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: 56px 1fr;
    grid-template-columns: 272px minmax(0, 1fr);
    grid-template-areas: "top top" "rail stage";
    height: 100vh;
  }
  @media (max-width: 640px) {
    .app {
      grid-template-columns: 216px minmax(0, 1fr);
    }
  }
  .topbar {
    grid-area: top;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 20px;
    border-bottom: 1px solid var(--haze-soft);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
  }
  .wordmark {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: 21px;
    letter-spacing: -0.02em;
    font-optical-sizing: auto;
  }
  .wordmark b {
    color: var(--eye);
    font-weight: 800;
  }
  .tagline {
    color: var(--dimmer);
    font-size: 12.5px;
  }
  .spacer {
    flex: 1;
  }
  .livepill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--eye-bright);
    background: rgba(255, 178, 62, 0.1);
    border: 1px solid rgba(255, 178, 62, 0.28);
    padding: 6px 12px 6px 10px;
    border-radius: var(--r-pill);
  }
  .livepill .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--eye);
    box-shadow: var(--glow);
    animation: owl-pulse 2s ease-in-out infinite;
  }
  .gear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--r-input);
    border: 1px solid transparent;
    background: transparent;
    color: var(--dim);
    cursor: pointer;
  }
  .gear:hover {
    color: var(--moon);
    background: var(--haze-soft);
  }
  .gear.active {
    color: var(--eye);
    border-color: var(--haze);
    background: var(--ink-card);
  }
  @media (max-width: 720px) {
    .tagline {
      display: none;
    }
  }
</style>
