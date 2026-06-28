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
