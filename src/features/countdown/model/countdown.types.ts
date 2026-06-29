import type { Duration } from "../../../shared/time/duration";
import type { EmptyPayload } from "../../../shared/payloads/empty";

export type CountdownState = "Idle" | "Running" | "Paused" | "Finished";

export type CountdownSnapshotDto = {
  id: number;
  label: string;
  duration: number;
  initial_duration: number;
  state: CountdownState;
  start_epoch_ms: number | null;
  target_epoch_ms: number | null;
};

export type CountdownSnapshot = {
  id: number;
  label: string;
  duration: Duration;
  initialDuration: Duration;
  state: CountdownState;
  start_epoch: Date | null;
  target_epoch: Date | null;
};

export type CountdownCommand =
  | "countdown_create"
  | "countdown_delete"
  | "countdown_list"
  | "countdown_start"
  | "countdown_pause"
  | "countdown_resume"
  | "countdown_reset"
  | "countdown_snapshot"
  | "set_overlay_config";

export type CountdownPayload =
  | EmptyPayload
  | CountdownIdPayload
  | CountdownCreatePayload
  | OverlayConfigPayload;

export type CountdownIdPayload = {
  id: number;
};

export type CountdownCreatePayload = {
  label: string;
  duration: number;
};

export type CountdownTickPayload = {
  id: number;
  label: string;
  remaining_ms: number;
};

/** Mirrors the Rust `OverlayConfig` struct (serde `camelCase`). */
export type OverlayConfig = {
  icon: string;
  showTimer: boolean;
  showProgress: boolean;
  fontSize: number;
  fontFamily: string;
  textColor: string;
  background: string;
  border: string;
  borderRadius: number;
  backdropFilter: boolean;
  boxShadow: string;
  iconSize: string;
  dividerColor: string;
  barBg: string;
  barFg: string;
  showHhMm: boolean;
};

export type OverlayConfigPayload = {
  id: number;
  config: OverlayConfig;
};
