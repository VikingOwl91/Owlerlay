<script lang="ts">
  import { countdownStore } from "../../features/countdown/state/countdown.store";
  import type { Duration } from "../../shared/time/duration";
  import CountdownDetail from "../../features/countdown/components/CountdownDetail.svelte";
  import GroupPanel from "../../features/overlay/components/GroupPanel.svelte";
  import OwlMark from "./OwlMark.svelte";
  import type { Subject } from "./types";

  let {
    subject,
    onCreated,
    onStartCreate,
  }: {
    subject: Subject;
    onCreated: () => void;
    onStartCreate: () => void;
  } = $props();

  let label = $state("");
  let hours = $state(0);
  let minutes = $state(0);
  let seconds = $state(0);

  function handleCreate() {
    if (!label.trim() && hours + minutes + seconds === 0) return;
    const duration: Duration = { hours, minutes, seconds, millis: 0 };
    void countdownStore.create(label.trim() || "Countdown", duration);
    label = "";
    hours = minutes = seconds = 0;
    onCreated();
  }
</script>

<main class="stage">
  {#if subject === "create"}
    <div class="detail narrow">
      <h1>New countdown</h1>
      <p class="sub">Name it and set how long it runs.</p>
      <form
        class="create"
        onsubmit={(e) => {
          e.preventDefault();
          handleCreate();
        }}
      >
        <input
          class="text"
          bind:value={label}
          placeholder="Label, e.g. Starting Soon"
          aria-label="Label"
        />
        <div class="hms">
          <input
            type="number"
            min="0"
            max="99"
            bind:value={hours}
            aria-label="Hours"
          />
          <span>:</span>
          <input
            type="number"
            min="0"
            max="59"
            bind:value={minutes}
            aria-label="Minutes"
          />
          <span>:</span>
          <input
            type="number"
            min="0"
            max="59"
            bind:value={seconds}
            aria-label="Seconds"
          />
        </div>
        <button type="submit" class="btn primary">Hatch countdown</button>
      </form>
    </div>
  {:else if subject === "group"}
    <GroupPanel />
  {:else if $countdownStore.selected}
    <CountdownDetail />
  {:else}
    <div class="placeholder">
      <OwlMark size={64} awake={false} />
      <h2>The roost is empty</h2>
      <p>Hatch your first countdown to put it on stream.</p>
      <button class="btn primary" onclick={onStartCreate}
        >＋ New countdown</button
      >
    </div>
  {/if}
</main>

<style>
  .stage {
    grid-area: stage;
    min-width: 0;
    overflow-y: auto;
    padding: 30px 36px;
  }
  @media (max-width: 640px) {
    .stage {
      padding: 24px 20px;
    }
  }
  .detail.narrow {
    max-width: 460px;
  }
  h1 {
    font-size: 26px;
  }
  .sub {
    color: var(--dim);
    font-size: 13.5px;
    margin: 4px 0 26px;
  }

  .create {
    display: flex;
    flex-direction: column;
    gap: 14px;
    align-items: flex-start;
  }
  .text {
    width: 100%;
    background: var(--ink-raised);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 12px 14px;
    color: var(--moon);
    font-size: 15px;
  }
  .text::placeholder {
    color: var(--dimmer);
  }
  .hms {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--dim);
    font-family: var(--font-mono);
  }
  .hms input {
    width: 72px;
    text-align: center;
    background: var(--ink-raised);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 12px 8px;
    color: var(--moon);
    font-family: var(--font-mono);
    font-size: 16px;
  }

  .btn {
    font-family: var(--font-body);
    font-weight: 700;
    font-size: 14.5px;
    border-radius: var(--r-pill);
    padding: 13px 24px;
    border: 1px solid transparent;
  }
  .btn.primary {
    background: var(--eye);
    color: #2a1c05;
    box-shadow: var(--glow);
  }
  .btn.primary:hover {
    background: var(--eye-bright);
  }

  .placeholder {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    text-align: center;
    color: var(--dim);
  }
  .placeholder h2 {
    font-size: 20px;
    color: var(--moon);
    margin-top: 8px;
  }
  .placeholder p {
    font-size: 14px;
    margin-bottom: 14px;
  }
</style>
