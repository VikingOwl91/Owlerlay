import { invokeCommand } from "../../../shared/tauri/invoke";
import type {
  CountdownCommand,
  CountdownPayload,
  OverlayConfig,
  CountdownSnapshot,
  CountdownSnapshotDto,
} from "../model/countdown.types";
import { mapSnapshotDtoToSnapshot } from "../model/countdown.mapper";
import { type Duration, durationToMillis } from "../../../shared/time/duration";

async function invokeCountdownCommand(
  command: CountdownCommand,
  payload: CountdownPayload,
): Promise<void> {
  await invokeCommand<void>(command, payload);
}

export async function createCountdown(
  label: string,
  duration: Duration,
): Promise<number> {
  return await invokeCommand("countdown_create", {
    label,
    duration: durationToMillis(duration),
  });
}

export async function listCountdowns(): Promise<CountdownSnapshot[]> {
  let snapshotsDto = await invokeCommand<CountdownSnapshotDto[]>(
    "countdown_list",
    {},
  );
  let snapshots: CountdownSnapshot[] = [];
  let snapshot: CountdownSnapshotDto;
  for (snapshot of snapshotsDto) {
    snapshots.push(mapSnapshotDtoToSnapshot(snapshot));
  }
  return snapshots;
}

export async function deleteCountdown(id: number): Promise<void> {
  await invokeCountdownCommand("countdown_delete", { id });
}

export async function startCountdown(id: number): Promise<void> {
  await invokeCountdownCommand("countdown_start", { id });
}

export async function resumeCountdown(id: number): Promise<void> {
  await invokeCountdownCommand("countdown_resume", { id });
}

export async function pauseCountdown(id: number): Promise<void> {
  await invokeCountdownCommand("countdown_pause", { id });
}

export async function resetCountdown(id: number): Promise<void> {
  await invokeCountdownCommand("countdown_reset", { id });
}

export async function fetchCountdownSnapshot(
  id: number,
): Promise<CountdownSnapshot> {
  return mapSnapshotDtoToSnapshot(
    await invokeCommand<CountdownSnapshotDto>("countdown_snapshot", { id }),
  );
}

export async function setOverlayConfig(
  id: number,
  config: OverlayConfig,
): Promise<void> {
  await invokeCommand<void>("set_overlay_config", { id, config });
}
