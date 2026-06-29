<script lang="ts">
  import { countdownStore } from "../state/countdown.store";
  import {
    formatClock,
    durationToMillis,
    type Duration,
  } from "../../../shared/time/duration";
  import EyePip from "../../../app/shell/EyePip.svelte";
  import AppearancePanel from "./AppearancePanel.svelte";

  const sel = $derived($countdownStore.selected!);
  const live = $derived($countdownStore.liveRemaining);

  const shown = $derived<Duration>(
    sel.state === "Running" && live ? live : sel.duration,
  );
  const parts = $derived(formatClock(shown).split(":"));
  const percent = $derived.by(() => {
    const initial = durationToMillis(sel.initialDuration);
    if (initial <= 0) return 0;
    return Math.max(
      0,
      Math.min(100, (durationToMillis(shown) / initial) * 100),
    );
  });
</script>

<div class="detail">
  <div class="head">
    <h1>{sel.label}</h1>
    <span class="chip" class:run={sel.state === "Running"}>
      <EyePip state={sel.state} size={9} />
      {sel.state}
    </span>
  </div>
  <p class="sub">Counts down from {formatClock(sel.initialDuration)}</p>

  <section class="readout" class:awake={sel.state === "Running"}>
    <div class="clock">
      <span>{parts[0]}</span><span class="sep">:</span><span>{parts[1]}</span
      ><span class="sep">:</span><span>{parts[2]}</span>
    </div>
    <div class="bar"><i style="width:{percent}%"></i></div>
  </section>

  <div class="transport">
    {#if sel.state === "Idle"}
      <button class="btn primary" onclick={countdownStore.startSelected}
        >▸ Start</button
      >
    {:else if sel.state === "Running"}
      <button class="btn primary" onclick={countdownStore.pauseSelected}
        >❙❙ Pause</button
      >
    {:else if sel.state === "Paused"}
      <button class="btn primary" onclick={countdownStore.resumeSelected}
        >▸ Resume</button
      >
    {/if}
    <button class="btn ghost" onclick={countdownStore.resetSelected}
      >↺ Reset</button
    >
    <button class="btn danger" onclick={countdownStore.deleteSelected}
      >Delete</button
    >
  </div>

  <AppearancePanel id={sel.id} />
</div>

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
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--dim);
    border: 1px solid var(--haze);
    background: var(--haze-soft);
    padding: 5px 11px;
    border-radius: var(--r-pill);
  }
  .chip.run {
    color: var(--eye-bright);
    border-color: rgba(255, 178, 62, 0.3);
    background: rgba(255, 178, 62, 0.08);
  }
  .sub {
    color: var(--dim);
    font-size: 13.5px;
    margin: 4px 0 26px;
  }

  .readout {
    container-type: inline-size;
    position: relative;
    overflow: hidden;
    background: var(--ink-raised);
    border: 1px solid var(--haze);
    border-radius: var(--r-card);
    padding: 38px 30px 30px;
    text-align: center;
  }
  .readout.awake {
    background:
      radial-gradient(
        120% 140% at 50% 0%,
        rgba(255, 178, 62, 0.07),
        transparent 60%
      ),
      var(--ink-raised);
  }
  .clock {
    font-family: var(--font-mono);
    font-weight: 600;
    /* size to the readout panel (container) so it never overflows the stage */
    font-size: clamp(30px, 13cqi, 92px);
    letter-spacing: 0.02em;
    line-height: 1;
    color: var(--dim);
    font-variant-numeric: tabular-nums;
  }
  .readout.awake .clock {
    color: var(--eye-bright);
    text-shadow: var(--glow-strong);
  }
  .readout.awake .sep {
    animation: owl-blink 2s step-end infinite;
  }
  .bar {
    height: 6px;
    border-radius: 999px;
    background: var(--haze);
    margin: 26px auto 0;
    max-width: 520px;
    overflow: hidden;
  }
  .bar > i {
    display: block;
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--talon), var(--eye));
  }
  .readout.awake .bar > i {
    box-shadow: var(--glow);
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
</style>
