//! Simulation timing model and clock scaling.
//!
//! The SimulationClock controls how fast simulated operations complete.
//! A scale of 1.0 is real-time, 0.0 is instant (for CI), and intermediate
//! values speed things up proportionally.

use std::time::Duration;

/// A clock that scales simulated durations for configurable-speed simulation.
#[derive(Debug, Clone)]
pub struct SimulationClock {
    /// Time scale factor: 1.0 = real-time, 0.0 = instant, 0.5 = 2x faster.
    scale: f64,
}

impl SimulationClock {
    /// Create a new simulation clock with the given scale factor.
    ///
    /// - `0.0` = instant (for CI/tests)
    /// - `0.1` = 10x faster than real-time
    /// - `1.0` = real-time
    pub fn new(scale: f64) -> Self {
        Self {
            scale: scale.max(0.0),
        }
    }

    /// Create an instant clock (scale = 0.0).
    pub fn instant() -> Self {
        Self::new(0.0)
    }

    /// Create a real-time clock (scale = 1.0).
    pub fn realtime() -> Self {
        Self::new(1.0)
    }

    /// The current scale factor.
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Whether this clock is instant (scale = 0).
    pub fn is_instant(&self) -> bool {
        self.scale == 0.0
    }

    /// Scale a duration according to the current time factor.
    pub fn scale_duration(&self, d: Duration) -> Duration {
        if self.scale == 0.0 {
            Duration::ZERO
        } else {
            d.mul_f64(self.scale)
        }
    }

    /// Sleep for a simulated duration, scaled by the clock factor.
    /// Returns immediately if scale is 0.
    pub async fn sleep(&self, real_duration: Duration) {
        let scaled = self.scale_duration(real_duration);
        if !scaled.is_zero() {
            tokio::time::sleep(scaled).await;
        }
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::instant()
    }
}

/// Per-generation timing constants for physical operations.
#[derive(Debug, Clone)]
pub struct TimingModel {
    /// Sustained native read rate (bytes/sec).
    pub native_read_rate: u64,
    /// Sustained native write rate (bytes/sec).
    pub native_write_rate: u64,
    /// Full rewind time (seconds).
    pub rewind_full_sec: f64,
    /// Cartridge load time (seconds).
    pub load_sec: f64,
    /// Cartridge unload time (seconds).
    pub unload_sec: f64,
    /// Time per wrap traversed during LOCATE (seconds).
    pub locate_per_wrap_sec: f64,
    /// Typical backhitch time (seconds).
    pub backhitch_sec: f64,
    /// Speed change penalty (seconds).
    pub speed_change_sec: f64,
    /// LTO-9 media optimization time (seconds).
    pub optimization_sec: f64,
}

impl TimingModel {
    /// Timing model for LTO-9.
    pub fn lto9() -> Self {
        Self {
            native_read_rate: 400_000_000,
            native_write_rate: 400_000_000,
            rewind_full_sec: 100.0,
            load_sec: 18.0,
            unload_sec: 18.0,
            locate_per_wrap_sec: 2.0,
            backhitch_sec: 2.0,
            speed_change_sec: 1.5,
            optimization_sec: 1200.0,
        }
    }

    /// Timing model for LTO-8.
    pub fn lto8() -> Self {
        Self {
            native_read_rate: 360_000_000,
            native_write_rate: 360_000_000,
            rewind_full_sec: 90.0,
            load_sec: 16.0,
            unload_sec: 16.0,
            locate_per_wrap_sec: 2.0,
            backhitch_sec: 2.0,
            speed_change_sec: 1.5,
            optimization_sec: 0.0,
        }
    }

    /// Timing model for LTO-7.
    pub fn lto7() -> Self {
        Self {
            native_read_rate: 300_000_000,
            native_write_rate: 300_000_000,
            rewind_full_sec: 80.0,
            load_sec: 15.0,
            unload_sec: 15.0,
            locate_per_wrap_sec: 2.0,
            backhitch_sec: 2.0,
            speed_change_sec: 1.5,
            optimization_sec: 0.0,
        }
    }

    /// Default timing model (LTO-9).
    pub fn default_for_generation(gen: crate::media::geometry::LtoGeneration) -> Self {
        use crate::media::geometry::LtoGeneration::*;
        match gen {
            Lto5 | Lto6 => Self::lto7(), // approximate
            Lto7 => Self::lto7(),
            Lto8 | Lto8M => Self::lto8(),
            Lto9 => Self::lto9(),
        }
    }
}
