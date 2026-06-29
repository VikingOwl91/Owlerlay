import { invokeCommand } from "../../shared/tauri/invoke";

/** Mirror of the Rust `RemoteSettings` (serde, snake_case). */
export interface RemoteSettings {
  /** Persisted desired state (what the toggle reflects). */
  enabled: boolean;
  /** Capability URL for the QR — present only when the server is LAN-bound. */
  url: string | null;
  token: string | null;
  /** Persisted flag differs from the running bind → restart needed. */
  restart_required: boolean;
}

export function getRemoteSettings(): Promise<RemoteSettings> {
  return invokeCommand<RemoteSettings>("remote_get_settings", {});
}

export function setRemoteEnabled(enabled: boolean): Promise<void> {
  return invokeCommand<void>("remote_set_enabled", { enabled });
}

export function regenerateRemoteToken(): Promise<RemoteSettings> {
  return invokeCommand<RemoteSettings>("remote_regenerate_token", {});
}
