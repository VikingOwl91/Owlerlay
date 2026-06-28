<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { countdownStore } from "../state/countdown.store";
  import { setOverlayConfig } from "../api/countdown.client";
  import { type Duration, formatDuration } from "../../../shared/time/duration";
  import { OVERLAY_SERVER_ORIGIN } from "../../../shared/overlay/origin";
  import GroupManager from "../../overlay/components/GroupManager.svelte";

  type OverlaySettings = {
    icon: string;
    textColor: string;
    bgColor: string;
    bgTransparent: boolean;
    showHHMM: boolean;
  };

  const DEFAULT_SETTINGS: OverlaySettings = {
    icon: "",
    textColor: "#ffffff",
    bgColor: "#000000",
    bgTransparent: true,
    showHHMM: false,
  };

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

  function updateSettings(id: number, patch: Partial<OverlaySettings>) {
    overlaySettings[id] = { ...getSettings(id), ...patch };
  }

  function pushConfig(id: number) {
    const s = getSettings(id);
    void setOverlayConfig(
      id,
      s.icon,
      s.textColor,
      s.bgTransparent ? "transparent" : s.bgColor,
      s.showHHMM,
    ).catch((error) => console.error(error));
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
  <article class="countdown-item-card">
    <details
      ontoggle={(e) => {
        if ((e.currentTarget as HTMLDetailsElement).open)
          countdownStore.select(item.id);
      }}
    >
      <summary>
        {#if getSettings(item.id).icon}
          <img
            src={`${OVERLAY_SERVER_ORIGIN}/static/icons/${getSettings(item.id).icon}`}
            alt={getSettings(item.id).icon}
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
            class="icon-btn {getSettings(item.id).icon === name
              ? 'selected'
              : ''}"
            onclick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              updateSettings(item.id, { icon: name });
              pushConfig(item.id);
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
            updateSettings(item.id, { icon: "" });
            pushConfig(item.id);
          }}
        >
          ✕
        </button>
      </div>

      <div class="overlay-colors">
        <label>
          Text
          <input
            type="color"
            value={getSettings(item.id).textColor}
            onchange={(e) => {
              updateSettings(item.id, { textColor: e.currentTarget.value });
              pushConfig(item.id);
            }}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={getSettings(item.id).bgTransparent}
            onchange={(e) => {
              updateSettings(item.id, {
                bgTransparent: e.currentTarget.checked,
              });
              pushConfig(item.id);
            }}
          />
          Transparent BG
        </label>
        {#if !getSettings(item.id).bgTransparent}
          <label>
            BG
            <input
              type="color"
              value={getSettings(item.id).bgColor}
              onchange={(e) => {
                updateSettings(item.id, { bgColor: e.currentTarget.value });
                pushConfig(item.id);
              }}
            />
          </label>
        {/if}
        <br />
      </div>
      <div>
        <label>
          <input
            type="checkbox"
            checked={getSettings(item.id).showHHMM}
            onchange={(e) => {
              updateSettings(item.id, { showHHMM: e.currentTarget.checked });
              pushConfig(item.id);
            }}
          />
          Show HH:MM
        </label>
      </div>
    </details>
  </article>
{/each}

<GroupManager
  countdowns={$countdownStore.items.map((i) => ({ id: i.id, label: i.label }))}
/>
