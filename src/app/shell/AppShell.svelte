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
  @media (max-width: 720px) {
    .tagline {
      display: none;
    }
  }
</style>
