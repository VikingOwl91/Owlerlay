<script lang="ts">
  import { onMount } from "svelte";
  import { setOverlayConfig } from "../api/countdown.client";
  import type { OverlayConfig } from "../model/countdown.types";
  import { OVERLAY_SERVER_ORIGIN } from "../../../shared/overlay/origin";

  let { id }: { id: number } = $props();

  // UI-facing settings; a few fields are split out from the wire `OverlayConfig`
  // for friendlier controls (transparent toggle, border width+colour, shadow
  // on/off, icon/font size in rem). `toConfig` composes them back.
  // Curated font choices. The overlay runs in OBS's browser, so each maps to a
  // CSS stack with broad system fallbacks rather than a single (maybe missing)
  // face. The control-room faces lead; fallbacks cover any OBS machine.
  const FONTS = {
    mono: {
      label: "Mono · Tabular",
      stack:
        'ui-monospace, "Spline Sans Mono", "JetBrains Mono", "SF Mono", Menlo, Consolas, monospace',
    },
    sans: {
      label: "Clean Sans",
      stack:
        '"Hanken Grotesk", "Inter", -apple-system, "Segoe UI", Roboto, system-ui, sans-serif',
    },
    display: {
      label: "Bold Display",
      stack:
        '"Bricolage Grotesque", "Archivo Black", "Arial Black", Impact, system-ui, sans-serif',
    },
    rounded: {
      label: "Rounded",
      stack:
        '"Quicksand", "Varela Round", "Nunito", "SF Pro Rounded", system-ui, sans-serif',
    },
    serif: {
      label: "Serif",
      stack: 'Georgia, "Playfair Display", "Times New Roman", serif',
    },
  } as const;
  type FontKey = keyof typeof FONTS;

  type OverlaySettings = {
    icon: string;
    showTimer: boolean;
    font: FontKey;
    fontSize: number;
    textColor: string;
    iconSize: number;
    bgTransparent: boolean;
    bgColor: string;
    borderWidth: number;
    borderColor: string;
    borderRadius: number;
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
    font: "mono",
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
      fontFamily: FONTS[s.font].stack,
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

  let icons = $state<string[]>([]);
  let overlaySettings = $state<Record<number, OverlaySettings>>({});

  function getSettings(n: number): OverlaySettings {
    return overlaySettings[n] ?? DEFAULT_SETTINGS;
  }
  function set(patch: Partial<OverlaySettings>) {
    overlaySettings[id] = { ...getSettings(id), ...patch };
    void setOverlayConfig(id, toConfig(overlaySettings[id])).catch((e) =>
      console.error(e),
    );
  }

  const s = $derived(getSettings(id));

  onMount(async () => {
    try {
      const res = await fetch(`${OVERLAY_SERVER_ORIGIN}/api/icons`);
      icons = await res.json();
    } catch {
      icons = [];
    }
  });
</script>

<details class="panel" open>
  <summary>Appearance <span class="chev">▾</span></summary>
  <div class="body">
    <div class="seg">Icon</div>
    <div class="icons">
      {#each icons as name (name)}
        <button
          type="button"
          class="iconbtn"
          class:sel={s.icon === name}
          onclick={() => set({ icon: name })}
        >
          <img
            src={`${OVERLAY_SERVER_ORIGIN}/static/icons/${name}`}
            alt={name}
          />
        </button>
      {/each}
      <button
        type="button"
        class="iconbtn none"
        class:sel={s.icon === ""}
        onclick={() => set({ icon: "" })}
        aria-label="No icon">✕</button
      >
    </div>

    <div class="seg">Display</div>
    <div class="grid">
      <div class="field">
        <label for="ap-st">Show timer</label>
        <input
          id="ap-st"
          type="checkbox"
          class="toggle"
          checked={s.showTimer}
          onchange={(e) => set({ showTimer: e.currentTarget.checked })}
        />
      </div>
      <div class="field">
        <label for="ap-hhmm">Show HH:MM</label>
        <input
          id="ap-hhmm"
          type="checkbox"
          class="toggle"
          checked={s.showHHMM}
          onchange={(e) => set({ showHHMM: e.currentTarget.checked })}
        />
      </div>
      <div class="field">
        <label for="ap-tc">Text colour</label>
        <input
          id="ap-tc"
          type="color"
          class="swatch"
          value={s.textColor}
          onchange={(e) => set({ textColor: e.currentTarget.value })}
        />
      </div>
      <div class="field">
        <label for="ap-fn">Font</label>
        <select
          id="ap-fn"
          class="select"
          value={s.font}
          onchange={(e) => set({ font: e.currentTarget.value as FontKey })}
        >
          {#each Object.entries(FONTS) as [key, f] (key)}
            <option value={key}>{f.label}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label for="ap-fs">Font size</label>
        <input
          id="ap-fs"
          type="number"
          class="num"
          min="0.5"
          max="6"
          step="0.25"
          value={s.fontSize}
          onchange={(e) => set({ fontSize: +e.currentTarget.value })}
        />
      </div>
      <div class="field">
        <label for="ap-is">Icon size</label>
        <input
          id="ap-is"
          type="number"
          class="num"
          min="0.5"
          max="8"
          step="0.25"
          value={s.iconSize}
          onchange={(e) => set({ iconSize: +e.currentTarget.value })}
        />
      </div>
    </div>

    <div class="seg">Container</div>
    <div class="grid">
      <div class="field">
        <label for="ap-bgt">Transparent BG</label>
        <input
          id="ap-bgt"
          type="checkbox"
          class="toggle"
          checked={s.bgTransparent}
          onchange={(e) => set({ bgTransparent: e.currentTarget.checked })}
        />
      </div>
      {#if !s.bgTransparent}
        <div class="field">
          <label for="ap-bg">Background</label>
          <input
            id="ap-bg"
            type="color"
            class="swatch"
            value={s.bgColor}
            onchange={(e) => set({ bgColor: e.currentTarget.value })}
          />
        </div>
      {/if}
      <div class="field">
        <label for="ap-bw">Border width</label>
        <input
          id="ap-bw"
          type="number"
          class="num"
          min="0"
          max="20"
          step="1"
          value={s.borderWidth}
          onchange={(e) => set({ borderWidth: +e.currentTarget.value })}
        />
      </div>
      {#if s.borderWidth > 0}
        <div class="field">
          <label for="ap-bc">Border colour</label>
          <input
            id="ap-bc"
            type="color"
            class="swatch"
            value={s.borderColor}
            onchange={(e) => set({ borderColor: e.currentTarget.value })}
          />
        </div>
      {/if}
      <div class="field">
        <label for="ap-br">Radius</label>
        <input
          id="ap-br"
          type="number"
          class="num"
          min="0"
          max="50"
          step="1"
          value={s.borderRadius}
          onchange={(e) => set({ borderRadius: +e.currentTarget.value })}
        />
      </div>
      <div class="field">
        <label for="ap-bf">Backdrop blur</label>
        <input
          id="ap-bf"
          type="checkbox"
          class="toggle"
          checked={s.backdropFilter}
          onchange={(e) => set({ backdropFilter: e.currentTarget.checked })}
        />
      </div>
      <div class="field">
        <label for="ap-sh">Shadow</label>
        <input
          id="ap-sh"
          type="checkbox"
          class="toggle"
          checked={s.boxShadow}
          onchange={(e) => set({ boxShadow: e.currentTarget.checked })}
        />
      </div>
    </div>

    <div class="seg">Progress bar</div>
    <div class="grid">
      <div class="field">
        <label for="ap-sp">Show progress</label>
        <input
          id="ap-sp"
          type="checkbox"
          class="toggle"
          checked={s.showProgress}
          onchange={(e) => set({ showProgress: e.currentTarget.checked })}
        />
      </div>
      {#if s.showProgress}
        <div class="field">
          <label for="ap-ff">Fill</label>
          <input
            id="ap-ff"
            type="color"
            class="swatch"
            value={s.barFg}
            onchange={(e) => set({ barFg: e.currentTarget.value })}
          />
        </div>
        <div class="field">
          <label for="ap-tk">Track</label>
          <input
            id="ap-tk"
            type="color"
            class="swatch"
            value={s.barBg}
            onchange={(e) => set({ barBg: e.currentTarget.value })}
          />
        </div>
        <div class="field">
          <label for="ap-dv">Divider</label>
          <input
            id="ap-dv"
            type="color"
            class="swatch"
            value={s.dividerColor}
            onchange={(e) => set({ dividerColor: e.currentTarget.value })}
          />
        </div>
      {/if}
    </div>
  </div>
</details>

<style>
  .panel {
    margin-top: 26px;
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-card);
    background: var(--ink-raised);
  }
  summary {
    display: flex;
    align-items: center;
    padding: 16px 20px;
    cursor: pointer;
    list-style: none;
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 15px;
  }
  summary::-webkit-details-marker {
    display: none;
  }
  .chev {
    margin-left: auto;
    color: var(--dim);
  }
  details[open] .chev {
    transform: rotate(180deg);
  }
  .body {
    padding: 0 20px 22px;
  }
  .seg {
    color: var(--dimmer);
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    margin: 16px 0 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 14px 40px;
    max-width: 640px;
  }
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .field label {
    color: var(--dim);
    font-size: 13.5px;
  }

  .icons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .iconbtn {
    width: 40px;
    height: 40px;
    border-radius: 9px;
    border: 1px solid var(--haze-soft);
    background: var(--ink);
    display: grid;
    place-items: center;
    color: var(--dim);
    font-size: 15px;
  }
  .iconbtn.sel {
    border-color: var(--eye);
    box-shadow: var(--glow);
  }
  .iconbtn img {
    width: 22px;
    height: 22px;
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

  .swatch {
    appearance: none;
    flex: none;
    width: 34px;
    height: 26px;
    border-radius: 7px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: none;
    padding: 0;
    cursor: pointer;
  }
  .swatch::-webkit-color-swatch-wrapper {
    padding: 2px;
  }
  .swatch::-webkit-color-swatch {
    border: none;
    border-radius: 5px;
  }

  .num {
    font-family: var(--font-mono);
    font-size: 13.5px;
    background: var(--ink);
    border: 1px solid var(--haze-soft);
    border-radius: 8px;
    padding: 6px 10px;
    color: var(--moon);
    width: 72px;
    text-align: center;
  }

  .select {
    font-size: 13.5px;
    background: var(--ink);
    border: 1px solid var(--haze-soft);
    border-radius: 8px;
    padding: 6px 10px;
    color: var(--moon);
    cursor: pointer;
  }
</style>
