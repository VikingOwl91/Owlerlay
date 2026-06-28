import type {
  CountdownSnapshot,
  CountdownSnapshotDto,
} from "./countdown.types";
import { type Duration, millisToDuration } from "../../../shared/time/duration";

export function mapSnapshotDtoToSnapshot(
  snapshotDto: CountdownSnapshotDto,
): CountdownSnapshot {
  const duration: Duration = millisToDuration(snapshotDto.duration);
  return {
    id: snapshotDto.id,
    label: snapshotDto.label,
    duration: duration,
    state: snapshotDto.state,
    start_epoch:
      snapshotDto.start_epoch_ms !== null
        ? new Date(snapshotDto.start_epoch_ms)
        : null,
    target_epoch:
      snapshotDto.target_epoch_ms !== null
        ? new Date(snapshotDto.target_epoch_ms)
        : null,
  };
}
