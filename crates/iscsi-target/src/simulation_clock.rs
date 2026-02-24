//! Simulation clock for controlling virtual device timing.
//!
//! The `SimulationClock` provides a shared, lock-free speed factor that scales
//! physical operation delays. A speed factor of 1.0 means realistic timing;
//! very large values (or `f64::INFINITY`) mean instant completion.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A shared clock that scales simulated durations for configurable-speed simulation.
///
/// Uses atomic operations for lock-free reads and writes, making it safe to
/// adjust from an API handler while SCSI command threads are sleeping.
pub struct SimulationClock {
    /// Speed factor stored as f64 bits for lock-free atomic access.
    /// 1.0 = realistic, >1 = faster, INFINITY = instant.
    speed_factor_bits: AtomicU64,
}

impl SimulationClock {
    /// Create a clock with the given speed factor.
    ///
    /// - `1.0` = realistic timing
    /// - `10.0` = 10x faster
    /// - `f64::INFINITY` = instant (no sleeping)
    pub fn new(speed_factor: f64) -> Self {
        Self {
            speed_factor_bits: AtomicU64::new(speed_factor.to_bits()),
        }
    }

    /// Create an instant clock (infinite speed factor).
    ///
    /// All operations complete without delay. This is the default,
    /// preserving backward-compatible behavior.
    pub fn instant() -> Self {
        Self::new(f64::INFINITY)
    }

    /// Create a realistic-speed clock (factor = 1.0).
    pub fn realistic() -> Self {
        Self::new(1.0)
    }

    /// Get the current speed factor.
    pub fn speed_factor(&self) -> f64 {
        f64::from_bits(self.speed_factor_bits.load(Ordering::Relaxed))
    }

    /// Set the speed factor. Safe to call from any thread.
    pub fn set_speed_factor(&self, factor: f64) {
        self.speed_factor_bits
            .store(factor.to_bits(), Ordering::Relaxed);
    }

    /// Sleep for a simulated duration, scaled by the speed factor.
    ///
    /// Actual sleep time = `real_duration / speed_factor`.
    /// Returns immediately if the speed factor is infinite or the
    /// resulting duration is negligible.
    ///
    /// This uses `std::thread::sleep` (blocking) and is intended to be
    /// called from within `spawn_blocking` tasks.
    pub fn sleep_sync(&self, real_duration: Duration) {
        let factor = self.speed_factor();
        if factor.is_infinite() || factor <= 0.0 {
            return;
        }
        let scaled = real_duration.div_f64(factor);
        if scaled.as_millis() < 1 {
            return;
        }
        std::thread::sleep(scaled);
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::instant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_does_not_sleep() {
        let clock = SimulationClock::instant();
        assert!(clock.speed_factor().is_infinite());
        // Should return immediately
        clock.sleep_sync(Duration::from_secs(100));
    }

    #[test]
    fn speed_factor_roundtrip() {
        let clock = SimulationClock::new(2.5);
        assert!((clock.speed_factor() - 2.5).abs() < f64::EPSILON);
        clock.set_speed_factor(10.0);
        assert!((clock.speed_factor() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_is_instant() {
        let clock = SimulationClock::default();
        assert!(clock.speed_factor().is_infinite());
    }
}
