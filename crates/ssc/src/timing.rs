//! Timing model for tape drive physical operations.
//!
//! Per-generation constants for simulating realistic drive latencies
//! (rewind, load, locate, backhitch, etc.).

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
