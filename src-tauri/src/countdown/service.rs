use crate::countdown::dto::CountdownSnapshotDto;
use crate::countdown::errors::CountdownError;
use crate::countdown::model::{Countdown, CountdownState};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

const MAX_COUNTDOWNS: usize = 10;

#[derive(Debug)]
pub struct CountdownService {
    countdowns: Mutex<HashMap<u64, Countdown>>,
    next_id: Mutex<u64>,
}

#[derive(Debug)]
pub struct CountdownSnapshot {
    pub id: u64,
    pub label: String,
    pub state: CountdownState,
    /// Remaining time at snapshot.
    pub duration: Duration,
    /// Configured countdown length.
    pub initial_duration: Duration,
    pub start_instant: Option<Instant>,
    pub target_instant: Option<Instant>,
}

pub struct TickResult {
    pub still_running: Vec<(u64, String, Duration)>,
    pub newly_finished: Vec<u64>,
}

/// Map one persisted DTO back to a live `Countdown`, re-anchoring time against
/// the current boot. We rebuild from wall-clock deltas rather than reconstruct
/// the original `Instant`s — subtracting a long downtime from a monotonic
/// `Instant` can underflow.
fn restore_countdown(dto: CountdownSnapshotDto, now: Instant, now_epoch_ms: u128) -> Countdown {
    let initial = Duration::from_millis(dto.initial_duration as u64);
    match dto.state {
        CountdownState::Idle => Countdown::new(dto.label, initial),
        CountdownState::Paused => Countdown::restore(
            dto.label,
            initial,
            CountdownState::Paused,
            Some(Duration::from_millis(dto.duration as u64)),
            None,
            None,
        ),
        CountdownState::Finished => Countdown::restore(
            dto.label,
            initial,
            CountdownState::Finished,
            Some(Duration::from_secs(0)),
            None,
            None,
        ),
        CountdownState::Running => {
            // Remaining wall-clock until the persisted target. If it already
            // elapsed (or there's no target), boot it Finished.
            let remaining_ms = dto
                .target_epoch_ms
                .map(|t| t.saturating_sub(now_epoch_ms))
                .unwrap_or(0);
            if remaining_ms == 0 {
                Countdown::restore(
                    dto.label,
                    initial,
                    CountdownState::Finished,
                    Some(Duration::from_secs(0)),
                    None,
                    None,
                )
            } else {
                let remaining = Duration::from_millis(remaining_ms as u64);
                // ponytail: start_timestamp is approximated as `now`. The model
                // never reads it (only target drives remaining_at); it's purely
                // cosmetic in snapshots.
                Countdown::restore(
                    dto.label,
                    initial,
                    CountdownState::Running,
                    None,
                    Some(now),
                    Some(now + remaining),
                )
            }
        }
    }
}

impl Default for CountdownService {
    fn default() -> Self {
        Self::new()
    }
}

impl CountdownService {
    pub fn new() -> Self {
        Self {
            countdowns: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
        }
    }

    /// Rebuild the service from persisted DTOs. `now_epoch_ms` is the boot
    /// wall-clock (from `ClockAnchor`); running countdowns are re-anchored
    /// against it so they keep counting across the restart, and any that
    /// expired during downtime come back `Finished`. `next_id` is derived as
    /// `max(id)+1` (gaps from past deletes don't matter — we only need to avoid
    /// colliding with a restored id).
    pub fn from_dtos(dtos: Vec<CountdownSnapshotDto>, now_epoch_ms: u128) -> Self {
        let now = Instant::now();
        let next_id = dtos.iter().map(|d| d.id + 1).max().unwrap_or(0);
        let mut countdowns = HashMap::with_capacity(dtos.len());
        for dto in dtos {
            countdowns.insert(dto.id, restore_countdown(dto, now, now_epoch_ms));
        }
        Self {
            countdowns: Mutex::new(countdowns),
            next_id: Mutex::new(next_id),
        }
    }

    pub async fn create_countdown(
        &self,
        label: String,
        duration: Duration,
    ) -> Result<u64, CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if countdowns.len() >= MAX_COUNTDOWNS {
            Err(CountdownError::MaxCountdownsReached)
        } else {
            if label.is_empty() {
                return Err(CountdownError::LabelNotFound);
            }
            if duration.as_millis() == 0 {
                return Err(CountdownError::InvalidDuration);
            }
            let mut next_id = self.next_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            countdowns.insert(id, Countdown::new(label, duration));
            Ok(id)
        }
    }

    pub async fn list_countdown(&self) -> Result<Vec<CountdownSnapshot>, CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if countdowns.is_empty() {
            return Ok(Vec::new());
        }
        let mut snapshots = Vec::new();
        for (id, countdown) in countdowns.iter_mut() {
            let now = Instant::now();
            countdown.sync_finished_at(now);
            snapshots.push(CountdownSnapshot {
                id: *id,
                label: countdown.label().to_string(),
                state: countdown.state(),
                duration: countdown.remaining_at(now),
                initial_duration: countdown.initial_duration(),
                start_instant: countdown.start_timestamp(),
                target_instant: countdown.target_timestamp(),
            })
        }
        snapshots.sort_by_key(|s| s.id);
        Ok(snapshots)
    }

    pub async fn delete_countdown(&self, id: u64) -> Result<(), CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.reset();
            countdowns.remove(&id);
            Ok(())
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn snapshot(
        &self,
        id: u64,
        now: Instant,
    ) -> Result<CountdownSnapshot, CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.sync_finished_at(now);
            Ok(CountdownSnapshot {
                id,
                label: countdown.label().to_string(),
                state: countdown.state(),
                duration: countdown.remaining_at(now),
                initial_duration: countdown.initial_duration(),
                start_instant: countdown.start_timestamp(),
                target_instant: countdown.target_timestamp(),
            })
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn start(&self, id: u64, now: Instant) -> Result<(), CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.start(now)
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn reset(&self, id: u64) -> Result<(), CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.reset();
            Ok(())
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn resume(&self, id: u64, now: Instant) -> Result<(), CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.resume(now)
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn pause(&self, id: u64, now: Instant) -> Result<(), CountdownError> {
        let mut countdowns = self.countdowns.lock().await;
        if let Some(countdown) = countdowns.get_mut(&id) {
            countdown.pause(now)
        } else {
            Err(CountdownError::IdNotFound)
        }
    }

    pub async fn tick(&self, now: Instant) -> TickResult {
        let mut countdowns = self.countdowns.lock().await;
        let mut still_running = vec![];
        let mut newly_finished = vec![];
        for (id, countdown) in countdowns.iter_mut() {
            if countdown.state() == CountdownState::Running {
                countdown.sync_finished_at(now);
                if countdown.is_finished() {
                    newly_finished.push(*id);
                } else {
                    still_running.push((
                        *id,
                        countdown.label().to_string(),
                        countdown.remaining_at(now),
                    ));
                }
            }
        }
        TickResult {
            still_running,
            newly_finished,
        }
    }
}
