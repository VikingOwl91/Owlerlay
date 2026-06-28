export type Duration = {
  hours: number;
  minutes: number;
  seconds: number;
  millis: number;
};

export function formatDuration(duration: Duration): string {
  return `${duration.hours.toString().padStart(2, "0")}:${duration.minutes.toString().padStart(2, "0")}:${duration.seconds.toString().padStart(2, "0")}.${duration.millis.toString().padStart(3, "0")}`;
}

export function millisToDuration(millis: number): Duration {
  return {
    hours: Math.floor(millis / 3600000),
    minutes: Math.floor((millis % 3600000) / 60000),
    seconds: Math.floor((millis % 60000) / 1000),
    millis: millis % 1000,
  };
}

export function durationToMillis(duration: Duration): number {
  return (
    duration.hours * 3600000 +
    duration.minutes * 60000 +
    duration.seconds * 1000 +
    duration.millis
  );
}
