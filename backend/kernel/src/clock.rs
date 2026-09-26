//! Clock port: every use case reads the time through it so tests stay deterministic.

use chrono::{DateTime, SubsecRound, Utc};

/// Source of the current time.
pub trait Clock: Send + Sync {
    /// Returns the current UTC time.
    fn now(&self) -> DateTime<Utc>;
}

/// [`Clock`] backed by the operating system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    /// Returns the current UTC time, truncated to microseconds to match
    /// PostgreSQL `timestamptz` precision.
    fn now(&self) -> DateTime<Utc> {
        Utc::now().trunc_subsecs(6)
    }
}

/// [`Clock`] frozen at a given instant (tests, replays).
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub DateTime<Utc>);

impl Clock for FixedClock {
    /// Returns the frozen instant.
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_uses_microsecond_precision() {
        assert_eq!(SystemClock.now().timestamp_subsec_nanos() % 1_000, 0);
    }

    #[test]
    fn fixed_clock_is_frozen() {
        let instant = Utc::now();
        assert_eq!(FixedClock(instant).now(), instant);
    }
}
