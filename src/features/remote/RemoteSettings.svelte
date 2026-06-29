<script lang="ts">
  import { onMount } from "svelte";
  import QRCode from "qrcode";
  import {
    getRemoteSettings,
    setRemoteEnabled,
    regenerateRemoteToken,
    type RemoteSettings,
  } from "./remote.client";

  let settings = $state<RemoteSettings | null>(null);
  let qrDataUrl = $state<string | null>(null);
  let copied = $state(false);
  let busy = $state(false);

  async function refresh() {
    settings = await getRemoteSettings();
    qrDataUrl = settings.url ? await renderQr(settings.url) : null;
  }

  function renderQr(url: string): Promise<string> {
    return QRCode.toDataURL(url, {
      margin: 1,
      width: 232,
      color: { dark: "#0d0f1aff", light: "#eef1f8ff" },
    });
  }

  async function toggle(enabled: boolean) {
    busy = true;
    try {
      await setRemoteEnabled(enabled);
      await refresh();
    } finally {
      busy = false;
    }
  }

  async function regenerate() {
    busy = true;
    try {
      settings = await regenerateRemoteToken();
      qrDataUrl = settings.url ? await renderQr(settings.url) : null;
    } finally {
      busy = false;
    }
  }

  async function copyUrl() {
    if (!settings?.url) return;
    await navigator.clipboard.writeText(settings.url);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }

  onMount(refresh);
</script>

<div class="panel">
  <h1>Phone remote</h1>
  <p class="sub">
    Control your timers from a phone or tablet on the same network.
  </p>

  {#if settings}
    <label class="toggle">
      <input
        type="checkbox"
        checked={settings.enabled}
        disabled={busy}
        onchange={(e) => toggle(e.currentTarget.checked)}
      />
      <span>Enable remote connect</span>
    </label>

    {#if settings.restart_required}
      <p class="notice">
        Restart Owlerlay to {settings.enabled ? "start" : "stop"} the LAN remote.
      </p>
    {/if}

    {#if settings.enabled && !settings.restart_required}
      {#if settings.url}
        <div class="pairing">
          {#if qrDataUrl}
            <img class="qr" src={qrDataUrl} alt="QR code to open the remote" />
          {/if}
          <p class="scan">Scan with your phone's camera.</p>
          <div class="urlrow">
            <code class="url">{settings.url}</code>
            <button class="mini" onclick={copyUrl}>
              {copied ? "Copied" : "Copy"}
            </button>
          </div>
          <button class="ghost" disabled={busy} onclick={regenerate}>
            Regenerate token
          </button>
          <p class="hint">
            Regenerating invalidates the old link on every connected device.
          </p>
        </div>
      {:else}
        <p class="notice">
          Couldn't determine this machine's LAN address. Check your network
          connection.
        </p>
      {/if}
    {/if}

    <p class="security">
      Anyone with the link can control your timers. Keep it to your own devices,
      and regenerate the token if it leaks.
    </p>
  {:else}
    <p class="sub">Loading…</p>
  {/if}
</div>

<style>
  .panel {
    max-width: 460px;
  }
  h1 {
    font-size: 26px;
  }
  .sub {
    color: var(--dim);
    font-size: 13.5px;
    margin: 4px 0 24px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 11px;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
  }
  .toggle input {
    width: 18px;
    height: 18px;
    accent-color: var(--eye);
  }
  .notice {
    margin: 14px 0 0;
    padding: 10px 13px;
    border-radius: var(--r-input);
    background: rgba(255, 178, 62, 0.1);
    border: 1px solid rgba(255, 178, 62, 0.28);
    color: var(--eye-bright);
    font-size: 13px;
  }
  .pairing {
    margin-top: 22px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  .qr {
    border-radius: var(--r-input);
    display: block;
  }
  .scan {
    color: var(--dim);
    font-size: 13px;
    margin: 0;
  }
  .urlrow {
    display: flex;
    gap: 8px;
    align-items: center;
    width: 100%;
  }
  .url {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    background: var(--ink-raised);
    border: 1px solid var(--haze-soft);
    border-radius: var(--r-input);
    padding: 9px 11px;
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--dim);
  }
  .mini {
    flex: none;
    padding: 9px 14px;
    border-radius: var(--r-input);
    border: 1px solid var(--haze-soft);
    background: transparent;
    color: var(--moon);
    font-size: 13px;
    font-weight: 600;
  }
  .mini:hover {
    border-color: var(--haze);
  }
  .ghost {
    margin-top: 4px;
    padding: 10px 16px;
    border-radius: var(--r-pill);
    border: 1px solid var(--haze-soft);
    background: transparent;
    color: var(--dim);
    font-size: 13.5px;
    font-weight: 600;
  }
  .ghost:hover {
    color: var(--moon);
    border-color: var(--haze);
  }
  .hint {
    color: var(--dimmer);
    font-size: 12px;
    margin: 0;
  }
  .security {
    margin-top: 26px;
    color: var(--dimmer);
    font-size: 12.5px;
    line-height: 1.5;
    border-top: 1px solid var(--haze-soft);
    padding-top: 16px;
  }
</style>
