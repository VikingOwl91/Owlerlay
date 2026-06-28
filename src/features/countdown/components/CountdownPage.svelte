<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { countdownStore } from "../state/countdown.store";
  import { setOverlayConfig } from "../api/countdown.client";
  import type { OverlayConfig } from "../model/countdown.types";
  import { type Duration, formatDuration } from "../../../shared/time/duration";
  import { OVERLAY_SERVER_ORIGIN } from "../../../shared/overlay/origin";
  import GroupManager from "../../overlay/components/GroupManager.svelte";

  // UI-facing settings. A few fields are split out from the wire `OverlayConfig`
  // for friendlier controls (transparent toggle, border width+color, shadow on/off,
  // icon/font size in rem); `toConfig` composes them back into the wire shape.
  type OverlaySettings = {
    icon: string;
    showTimer: boolean;
    fontSize: number; // rem
    textColor: string;
    iconSize: number; // rem
    bgTransparent: boolean;
    bgColor: string;
    borderWidth: number; // px; 0 = no border
    borderColor: string;
    borderRadius: number; // px
    backdropFilter: boolean;
    boxShadow: boolean;
    showProgress: boolean;
    barFg: string;
    barBg: string;
    dividerColor: string;
    showHHMM: boolean;
  };

  const DEFAULT_SETTINGS: OverlaySettings = {
    icon: "",
    showTimer: true,
    fontSize: 2,
    textColor: "#ffffff",
    iconSize: 2,
    bgTransparent: true,
    bgColor: "#000000",
    borderWidth: 0,
    borderColor: "#ffffff",
    borderRadius: 8,
    backdropFilter: false,
    boxShadow: false,
    showProgress: false,
    barFg: "#4ade80",
    barBg: "#333333",
    dividerColor: "#ffffff",
    showHHMM: false,
  };

  function toConfig(s: OverlaySettings): OverlayConfig {
    return {
      icon: s.icon,
      showTimer: s.showTimer,
      showProgress: s.showProgress,
      fontSize: s.fontSize,
      textColor: s.textColor,
      background: s.bgTransparent ? "transparent" : s.bgColor,
      border:
        s.borderWidth > 0
          ? `${s.borderWidth}px solid ${s.borderColor}`
          : "none",
      borderRadius: s.borderRadius,
      backdropFilter: s.backdropFilter,
      boxShadow: s.boxShadow ? "0 4px 12px rgba(0,0,0,0.4)" : "",
      iconSize: `${s.iconSize}rem`,
      dividerColor: s.dividerColor,
      barBg: s.barBg,
      barFg: s.barFg,
      showHhMm: s.showHHMM,
    };
  }

  let label = $state("");
  let hours = $state(0);
  let minutes = $state(0);
  let seconds = $state(0);
  let icons = $state<string[]>([]);
  let overlaySettings = $state<Record<number, OverlaySettings>>({});
  let cleanup: (() => void) | null = null;

  function getSettings(id: number): OverlaySettings {
    return overlaySettings[id] ?? DEFAULT_SETTINGS;
  }

  function pushConfig(id: number) {
    void setOverlayConfig(id, toConfig(getSettings(id))).catch((error) =>
      console.error(error),
    );
  }

  function set(id: number, patch: Partial<OverlaySettings>) {
    overlaySettings[id] = { ...getSettings(id), ...patch };
    pushConfig(id);
  }

  function handleCreate() {
    const duration: Duration = { hours, minutes, seconds, millis: 0 };
    void countdownStore.create(label, duration);
    label = "";
    hours = 0;
    minutes = 0;
    seconds = 0;
  }

  function selectAndRun(id: number, action: () => Promise<void>) {
    countdownStore.select(id);
    void action();
  }

  onMount(async () => {
    void countdownStore.loadList();
    cleanup = await countdownStore.initStoreListeners();
    try {
      const res = await fetch(`${OVERLAY_SERVER_ORIGIN}/api/icons`);
      icons = await res.json();
    } catch {
      icons = [];
    }
  });

  onDestroy(() => cleanup?.());
</script>

{#if $countdownStore.loading}<p aria-busy="true">Loading...</p>{/if}
{#if $countdownStore.error}<p class="countdown-error">
    {$countdownStore.error}
  </p>{/if}

<article>
  <header>New Countdown</header>
  <input bind:value={label} placeholder="Label" required />
  <!-- svelte-ignore a11y_no_redundant_roles -->
  <fieldset role="group">
    <input
      aria-label="Hours"
      bind:value={hours}
      max="99"
      min="0"
      placeholder="hh"
      type="number"
    />
    <input
      aria-label="Minutes"
      bind:value={minutes}
      max="59"
      min="0"
      placeholder="mm"
      type="number"
    />
    <input
      aria-label="Seconds"
      bind:value={seconds}
      max="59"
      min="0"
      placeholder="ss"
      type="number"
    />
    <button type="button" onclick={handleCreate}>Create</button>
  </fieldset>
</article>

{#each $countdownStore.items as item (item.id)}
  {@const s = getSettings(item.id)}
  <article class="countdown-item-card">
    <details
      ontoggle={(e) => {
        if ((e.currentTarget as HTMLDetailsElement).open)
          countdownStore.select(item.id);
      }}
    >
      <summary>
        {#if s.icon}
          <img
            src={`${OVERLAY_SERVER_ORIGIN}/static/icons/${s.icon}`}
            alt={s.icon}
            style="width:1.2em;height:1.2em;vertical-align:middle;margin-right:0.3em;"
          />
        {/if}
        {item.label}
        <mark data-state={item.state}>{item.state}</mark>
      </summary>

      <p class="timer-display">
        {#if $countdownStore.selectedId === item.id && $countdownStore.liveRemaining}
          {formatDuration($countdownStore.liveRemaining)}
        {:else}
          {formatDuration(item.duration)}
        {/if}
      </p>

      <div class="countdown-actions">
        {#if item.state === "Idle"}
          <button
            type="button"
            onclick={() => selectAndRun(item.id, countdownStore.startSelected)}
            >Start</button
          >
        {:else if item.state === "Running"}
          <button
            type="button"
            onclick={() => selectAndRun(item.id, countdownStore.pauseSelected)}
            >Pause</button
          >
        {:else if item.state === "Paused"}
          <button
            type="button"
            onclick={() => selectAndRun(item.id, countdownStore.resumeSelected)}
            >Resume</button
          >
        {/if}
        <button
          type="button"
          class="secondary"
          onclick={() => selectAndRun(item.id, countdownStore.resetSelected)}
          >Reset</button
        >
        <button
          type="button"
          class="secondary contrast"
          onclick={() => selectAndRun(item.id, countdownStore.deleteSelected)}
          >Delete</button
        >
      </div>

      <hr />

      <p><small>Icon</small></p>
      <div class="icon-picker">
        {#each icons as name (name)}
          <button
            type="button"
            class="icon-btn {s.icon === name ? 'selected' : ''}"
            onclick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              set(item.id, { icon: name });
            }}
          >
            <img
              src={`${OVERLAY_SERVER_ORIGIN}/static/icons/${name}`}
              alt={name}
            />
          </button>
        {/each}
        <button
          type="button"
          class="icon-btn"
          onclick={(e) => {
            e.stopPropagation();
            e.preventDefault();
            set(item.id, { icon: "" });
          }}
        >
          ✕
        </button>
      </div>

      <p><small>Display</small></p>
      <div class="overlay-colors">
        <label>
          <input
            type="checkbox"
            checked={s.showTimer}
            onchange={(e) =>
              set(item.id, { showTimer: e.currentTarget.checked })}
          />
          Show timer
        </label>
        <label>
          Text
          <input
            type="color"
            value={s.textColor}
            onchange={(e) => set(item.id, { textColor: e.currentTarget.value })}
          />
        </label>
        <label>
          Font
          <input
            type="number"
            min="0.5"
            max="6"
            step="0.25"
            value={s.fontSize}
            onchange={(e) => set(item.id, { fontSize: +e.currentTarget.value })}
          />
        </label>
        <label>
          Icon size
          <input
            type="number"
            min="0.5"
            max="8"
            step="0.25"
            value={s.iconSize}
            onchange={(e) => set(item.id, { iconSize: +e.currentTarget.value })}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={s.showHHMM}
            onchange={(e) =>
              set(item.id, { showHHMM: e.currentTarget.checked })}
          />
          Show HH:MM
        </label>
      </div>

      <p><small>Container</small></p>
      <div class="overlay-colors">
        <label>
          <input
            type="checkbox"
            checked={s.bgTransparent}
            onchange={(e) =>
              set(item.id, { bgTransparent: e.currentTarget.checked })}
          />
          Transparent BG
        </label>
        {#if !s.bgTransparent}
          <label>
            BG
            <input
              type="color"
              value={s.bgColor}
              onchange={(e) => set(item.id, { bgColor: e.currentTarget.value })}
            />
          </label>
        {/if}
        <label>
          Border
          <input
            type="number"
            min="0"
            max="20"
            step="1"
            value={s.borderWidth}
            onchange={(e) =>
              set(item.id, { borderWidth: +e.currentTarget.value })}
          />
        </label>
        {#if s.borderWidth > 0}
          <label>
            Color
            <input
              type="color"
              value={s.borderColor}
              onchange={(e) =>
                set(item.id, { borderColor: e.currentTarget.value })}
            />
          </label>
        {/if}
        <label>
          Radius
          <input
            type="number"
            min="0"
            max="50"
            step="1"
            value={s.borderRadius}
            onchange={(e) =>
              set(item.id, { borderRadius: +e.currentTarget.value })}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={s.backdropFilter}
            onchange={(e) =>
              set(item.id, { backdropFilter: e.currentTarget.checked })}
          />
          Backdrop blur
        </label>
        <label>
          <input
            type="checkbox"
            checked={s.boxShadow}
            onchange={(e) =>
              set(item.id, { boxShadow: e.currentTarget.checked })}
          />
          Shadow
        </label>
      </div>

      <p><small>Progress bar</small></p>
      <div class="overlay-colors">
        <label>
          <input
            type="checkbox"
            checked={s.showProgress}
            onchange={(e) =>
              set(item.id, { showProgress: e.currentTarget.checked })}
          />
          Show progress
        </label>
        {#if s.showProgress}
          <label>
            Fill
            <input
              type="color"
              value={s.barFg}
              onchange={(e) => set(item.id, { barFg: e.currentTarget.value })}
            />
          </label>
          <label>
            Track
            <input
              type="color"
              value={s.barBg}
              onchange={(e) => set(item.id, { barBg: e.currentTarget.value })}
            />
          </label>
          <label>
            Divider
            <input
              type="color"
              value={s.dividerColor}
              onchange={(e) =>
                set(item.id, { dividerColor: e.currentTarget.value })}
            />
          </label>
        {/if}
      </div>
    </details>
  </article>
{/each}

<GroupManager
  countdowns={$countdownStore.items.map((i) => ({ id: i.id, label: i.label }))}
/>
