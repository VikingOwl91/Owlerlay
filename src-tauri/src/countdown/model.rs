use crate::countdown::errors::CountdownError;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CountdownState {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Countdown {
    pub label: String,
    initial_duration: Duration,
    remaining_duration_stored: Option<Duration>,
    state: CountdownState,
    start_timestamp: Option<Instant>,
    target_timestamp: Option<Instant>,
}

impl Countdown {
    pub fn new(label: impl Into<String>, duration: Duration) -> Self {
        Self {
            label: label.into(),
            initial_duration: duration,
            remaining_duration_stored: None,
            state: CountdownState::Idle,
            start_timestamp: None,
            target_timestamp: None,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    /// The configured countdown length, independent of how much time is left.
    pub fn initial_duration(&self) -> Duration {
        self.initial_duration
    }

    pub fn state(&self) -> CountdownState {
        self.state
    }

    pub fn start_timestamp(&self) -> Option<Instant> {
        self.start_timestamp
    }

    pub fn target_timestamp(&self) -> Option<Instant> {
        self.target_timestamp
    }

    pub fn remaining_at(&self, now: Instant) -> Duration {
        match self.state {
            CountdownState::Idle => self.initial_duration,
            CountdownState::Running => self
                .target_timestamp
                .map(|target| target.saturating_duration_since(now))
                .unwrap_or(self.initial_duration),
            CountdownState::Paused => self
                .remaining_duration_stored
                .unwrap_or(self.initial_duration),
            CountdownState::Finished => Duration::from_secs(0),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.state == CountdownState::Finished
    }

    pub fn start(&mut self, now: Instant) -> Result<(), CountdownError> {
        match self.state {
            CountdownState::Idle => {
                let target = now
                    .checked_add(self.initial_duration)
                    .ok_or(CountdownError::TimeOverflow)?;
                self.start_timestamp = Some(now);
                self.target_timestamp = Some(target);
                self.remaining_duration_stored = None;
                self.state = CountdownState::Running;
                Ok(())
            }
            CountdownState::Paused => {
                let remaining =
                    self.remaining_duration_stored
                        .ok_or(CountdownError::InvalidTransition {
                            from: self.state,
                            action: "start",
                        })?;

                if remaining.is_zero() {
                    self.mark_finished();
                    return Ok(());
                }

                let target = now
                    .checked_add(remaining)
                    .ok_or(CountdownError::TimeOverflow)?;
                self.start_timestamp = Some(now);
                self.target_timestamp = Some(target);
                self.remaining_duration_stored = None;
                self.state = CountdownState::Running;
                Ok(())
            }
            _ => Err(CountdownError::InvalidTransition {
                from: self.state,
                action: "start",
            }),
        }
    }

    pub fn pause(&mut self, now: Instant) -> Result<(), CountdownError> {
        if self.state != CountdownState::Running {
            return Err(CountdownError::InvalidTransition {
                from: self.state,
                action: "pause",
            });
        }

        let remaining = self.remaining_at(now);
        if remaining.is_zero() {
            self.mark_finished();
            return Ok(());
        }

        self.remaining_duration_stored = Some(remaining);
        self.start_timestamp = None;
        self.target_timestamp = None;
        self.state = CountdownState::Paused;
        Ok(())
    }

    pub fn resume(&mut self, now: Instant) -> Result<(), CountdownError> {
        if self.state != CountdownState::Paused {
            return Err(CountdownError::InvalidTransition {
                from: self.state,
                action: "resume",
            });
        }

        self.start(now)
    }

    pub fn reset(&mut self) {
        self.state = CountdownState::Idle;
        self.remaining_duration_stored = None;
        self.start_timestamp = None;
        self.target_timestamp = None;
    }

    pub fn sync_finished_at(&mut self, now: Instant) {
        if self.state == CountdownState::Running && self.remaining_at(now).is_zero() {
            self.mark_finished();
        }
    }

    fn mark_finished(&mut self) {
        self.state = CountdownState::Finished;
        self.remaining_duration_stored = Some(Duration::from_secs(0));
        self.start_timestamp = None;
        self.target_timestamp = None;
    }
}
