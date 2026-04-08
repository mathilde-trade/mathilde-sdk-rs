use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::error::SdkError;

const NANOS_PER_SEC: u128 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExponentialBackoffConfig {
    pub initial_delay: Duration,
    pub multiplier: u32,
    pub max_delay: Duration,
    pub max_attempts: Option<u32>,
    pub jitter_ratio: f64,
}

impl ExponentialBackoffConfig {
    pub fn validate(self) -> Result<Self, SdkError> {
        if self.initial_delay.is_zero() {
            return Err(SdkError::request_build(
                "ws recovery initial_delay must be > 0",
            ));
        }

        if self.multiplier < 1 {
            return Err(SdkError::request_build(
                "ws recovery multiplier must be >= 1",
            ));
        }

        if self.max_delay < self.initial_delay {
            return Err(SdkError::request_build(
                "ws recovery max_delay must be >= initial_delay",
            ));
        }

        if !(0.0..=1.0).contains(&self.jitter_ratio) {
            return Err(SdkError::request_build(
                "ws recovery jitter_ratio must be in [0.0, 1.0]",
            ));
        }

        Ok(self)
    }
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(250),
            multiplier: 2,
            max_delay: Duration::from_secs(10),
            max_attempts: None,
            jitter_ratio: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffDelay {
    pub attempt: u32,
    pub base_delay: Duration,
    pub min_delay: Duration,
    pub max_delay: Duration,
}

impl BackoffDelay {
    pub fn select_delay(self) -> Duration {
        if self.min_delay == self.max_delay {
            return self.base_delay;
        }

        let min_nanos = duration_to_nanos(self.min_delay);
        let max_nanos = duration_to_nanos(self.max_delay);
        let span = max_nanos.saturating_sub(min_nanos);
        if span == 0 {
            return self.min_delay;
        }

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(duration_to_nanos)
            .unwrap_or(0);
        let selected = min_nanos.saturating_add(seed % span.saturating_add(1));
        nanos_to_duration(selected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconnectBackoff {
    config: ExponentialBackoffConfig,
    next_attempt: u32,
}

impl ReconnectBackoff {
    pub fn new(config: ExponentialBackoffConfig) -> Result<Self, SdkError> {
        Ok(Self {
            config: config.validate()?,
            next_attempt: 1,
        })
    }

    pub fn config(&self) -> ExponentialBackoffConfig {
        self.config
    }

    pub fn next_attempt(&self) -> u32 {
        self.next_attempt
    }

    pub fn reset(&mut self) {
        self.next_attempt = 1;
    }

    pub fn next_delay(&mut self) -> Option<BackoffDelay> {
        if let Some(max_attempts) = self.config.max_attempts {
            if self.next_attempt > max_attempts {
                return None;
            }
        }

        let attempt = self.next_attempt;
        self.next_attempt = self.next_attempt.saturating_add(1);

        let base_delay = self.base_delay_for_attempt(attempt);
        let (min_delay, max_delay) = jitter_bounds(base_delay, self.config.jitter_ratio);

        Some(BackoffDelay {
            attempt,
            base_delay,
            min_delay,
            max_delay,
        })
    }

    pub fn next_sleep_duration(&mut self) -> Option<Duration> {
        self.next_delay().map(BackoffDelay::select_delay)
    }

    fn base_delay_for_attempt(&self, attempt: u32) -> Duration {
        let mut delay = self.config.initial_delay;

        for _ in 1..attempt {
            delay = saturating_mul_duration(delay, self.config.multiplier);
            if delay >= self.config.max_delay {
                return self.config.max_delay;
            }
        }

        delay.min(self.config.max_delay)
    }
}

fn jitter_bounds(base_delay: Duration, jitter_ratio: f64) -> (Duration, Duration) {
    if jitter_ratio == 0.0 {
        return (base_delay, base_delay);
    }

    let base_nanos = duration_to_nanos(base_delay);
    let spread_nanos = (base_nanos as f64 * jitter_ratio).round() as u128;
    let min_nanos = base_nanos.saturating_sub(spread_nanos);
    let max_nanos = base_nanos.saturating_add(spread_nanos);

    (nanos_to_duration(min_nanos), nanos_to_duration(max_nanos))
}

fn saturating_mul_duration(duration: Duration, multiplier: u32) -> Duration {
    nanos_to_duration(duration_to_nanos(duration).saturating_mul(multiplier as u128))
}

fn duration_to_nanos(duration: Duration) -> u128 {
    duration.as_secs() as u128 * NANOS_PER_SEC + duration.subsec_nanos() as u128
}

fn nanos_to_duration(nanos: u128) -> Duration {
    let secs = (nanos / NANOS_PER_SEC).min(u64::MAX as u128) as u64;
    let subsec_nanos = (nanos % NANOS_PER_SEC) as u32;
    Duration::new(secs, subsec_nanos)
}
