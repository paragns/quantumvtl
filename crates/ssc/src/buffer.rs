//! Drive buffer simulation and speed governing.
//!
//! Models the internal buffer and digital speed matching (DSM) of a real LTO
//! tape drive. This is a **simulation overlay**: actual file I/O happens
//! synchronously while buffer fill levels, timing delays, and speed selection
//! are simulated on top.

use std::time::{Duration, Instant};

use serde::Serialize;

use crate::media::geometry::TapeGeometry;
use iscsi_target::SimulationClock;
use crate::timing::TimingModel;

// ── Speed table ─────────────────────────────────────────────────────────────

/// A single discrete tape speed level.
#[derive(Debug, Clone)]
pub struct SpeedEntry {
    /// Speed index: 1 (fastest) through num_speeds (slowest).
    pub index: u8,
    /// Fraction of maximum tape velocity at this speed.
    pub velocity_fraction: f64,
    /// Native byte rate (bytes/sec) at this speed.
    pub native_rate: u64,
}

/// Pre-computed table of discrete tape speeds for a generation.
#[derive(Debug, Clone)]
pub struct SpeedTable {
    entries: Vec<SpeedEntry>,
}

impl SpeedTable {
    /// Build a speed table from tape geometry. Linearly spaces `num_speeds`
    /// levels from max rate down to max_rate / num_speeds.
    pub fn for_geometry(geometry: &TapeGeometry) -> Self {
        let n = geometry.num_speeds as usize;
        let max_rate = geometry.sustained_rate_bytes_sec;
        let mut entries = Vec::with_capacity(n);
        for i in 0..n {
            let index = (i + 1) as u8;
            // Speed 1 = fastest (full rate), speed N = slowest (1/N of max)
            let velocity_fraction = (n - i) as f64 / n as f64;
            let native_rate = (max_rate as f64 * velocity_fraction) as u64;
            entries.push(SpeedEntry {
                index,
                velocity_fraction,
                native_rate,
            });
        }
        Self { entries }
    }

    /// Pick the fastest speed whose native rate <= effective_host_rate.
    /// If host is slower than all speeds, returns the slowest.
    pub fn select_speed(&self, effective_host_rate: u64) -> &SpeedEntry {
        // entries are sorted fastest to slowest
        for entry in &self.entries {
            if entry.native_rate <= effective_host_rate {
                return entry;
            }
        }
        self.slowest()
    }

    pub fn fastest(&self) -> &SpeedEntry {
        &self.entries[0]
    }

    pub fn slowest(&self) -> &SpeedEntry {
        self.entries.last().unwrap_or(&self.entries[0])
    }

    pub fn by_index(&self, index: u8) -> Option<&SpeedEntry> {
        self.entries.iter().find(|e| e.index == index)
    }

    /// Clamp a speed index one step faster (lower index).
    pub fn one_faster(&self, current: u8) -> &SpeedEntry {
        let idx = current.saturating_sub(1).max(1);
        self.by_index(idx).unwrap_or(self.fastest())
    }

    /// Clamp a speed index one step slower (higher index).
    pub fn one_slower(&self, current: u8) -> &SpeedEntry {
        let max_idx = self.entries.last().map(|e| e.index).unwrap_or(1);
        let idx = (current + 1).min(max_idx);
        self.by_index(idx).unwrap_or(self.slowest())
    }

    pub fn num_speeds(&self) -> usize {
        self.entries.len()
    }
}

// ── Buffer phase ────────────────────────────────────────────────────────────

/// Current operational phase of the drive buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferPhase {
    /// No active I/O — buffer idle.
    Idle,
    /// Host is writing; buffer fills from host, drains to tape.
    Writing,
    /// Host is reading; tape fills cache, host drains it.
    Reading,
    /// Final flush: draining remaining write buffer to tape.
    Flushing,
}

impl BufferPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Writing => "writing",
            Self::Reading => "reading",
            Self::Flushing => "flushing",
        }
    }
}

// ── DriveBuffer ─────────────────────────────────────────────────────────────

/// Core buffer simulation state.
pub struct DriveBuffer {
    // Config
    capacity: usize,
    speed_table: SpeedTable,
    timing: TimingModel,

    // Write buffer state (host fills, tape drains)
    write_buffered_bytes: usize,
    write_buffered_objects: u32,

    // Read cache state (tape fills, host drains)
    read_cache_bytes: usize,
    read_cache_objects: u32,

    // Speed governor
    current_speed: SpeedEntry,
    host_rate_ewma: f64,
    ewma_alpha: f64,
    last_io_time: Option<Instant>,

    // Timing
    last_tick: Instant,
    phase: BufferPhase,

    // Metrics (per-mount, reset on media load)
    speed_change_count: u32,
    backhitch_count: u32,
    speed_time_secs: Vec<f64>,
    total_host_bytes: u64,
    total_tape_bytes: u64,
    high_water_mark_bytes: usize,
    stall_time_secs: f64,
    write_cycle_count: u32,
    read_cycle_count: u32,
}

impl DriveBuffer {
    /// Create a new buffer simulation for a tape with the given geometry.
    pub fn new(geometry: &TapeGeometry, timing: TimingModel) -> Self {
        let speed_table = SpeedTable::for_geometry(geometry);
        let current_speed = speed_table.fastest().clone();
        let num_speeds = speed_table.num_speeds();
        Self {
            capacity: geometry.buffer_size_bytes,
            speed_table,
            timing,
            write_buffered_bytes: 0,
            write_buffered_objects: 0,
            read_cache_bytes: 0,
            read_cache_objects: 0,
            current_speed,
            host_rate_ewma: 0.0,
            ewma_alpha: 0.3,
            last_io_time: None,
            last_tick: Instant::now(),
            phase: BufferPhase::Idle,
            speed_change_count: 0,
            backhitch_count: 0,
            speed_time_secs: vec![0.0; num_speeds],
            total_host_bytes: 0,
            total_tape_bytes: 0,
            high_water_mark_bytes: 0,
            stall_time_secs: 0.0,
            write_cycle_count: 0,
            read_cycle_count: 0,
        }
    }

    /// Tick the simulation forward. Call at the start of every command and
    /// from the background ticker. Drains write buffer or fills read cache
    /// at the current speed's rate.
    pub fn tick(&mut self, clock: &SimulationClock) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if clock.speed_factor().is_infinite() {
            // In instant mode, drain/fill everything immediately
            match self.phase {
                BufferPhase::Writing | BufferPhase::Flushing => {
                    self.total_tape_bytes += self.write_buffered_bytes as u64;
                    self.write_buffered_bytes = 0;
                    self.write_buffered_objects = 0;
                    if self.phase == BufferPhase::Flushing {
                        self.phase = BufferPhase::Idle;
                    }
                }
                BufferPhase::Reading => {
                    // Fill read cache up to capacity
                    self.read_cache_bytes = self.capacity;
                }
                BufferPhase::Idle => {}
            }
            return;
        }

        let elapsed_secs = elapsed.as_secs_f64() * clock.speed_factor();
        if elapsed_secs <= 0.0 {
            return;
        }

        let drain_bytes = (self.current_speed.native_rate as f64 * elapsed_secs) as usize;

        // Track time at current speed
        let speed_idx = (self.current_speed.index as usize).saturating_sub(1);
        if speed_idx < self.speed_time_secs.len() {
            self.speed_time_secs[speed_idx] += elapsed_secs;
        }

        match self.phase {
            BufferPhase::Writing => {
                let drained = drain_bytes.min(self.write_buffered_bytes);
                self.write_buffered_bytes -= drained;
                self.total_tape_bytes += drained as u64;
                // Decrease object count proportionally
                if drained > 0 && self.write_buffered_bytes == 0 {
                    self.write_buffered_objects = 0;
                }
            }
            BufferPhase::Flushing => {
                let drained = drain_bytes.min(self.write_buffered_bytes);
                self.write_buffered_bytes -= drained;
                self.total_tape_bytes += drained as u64;
                if self.write_buffered_bytes == 0 {
                    self.write_buffered_objects = 0;
                    self.phase = BufferPhase::Idle;
                }
            }
            BufferPhase::Reading => {
                // Tape fills read cache
                let filled = drain_bytes.min(self.capacity - self.read_cache_bytes);
                self.read_cache_bytes += filled;
            }
            BufferPhase::Idle => {}
        }
    }

    /// Record a host write into the buffer. Updates EWMA and speed governor.
    /// Returns a stall duration if the buffer is full (caller sleeps in
    /// realtime mode), or `Duration::ZERO` if there's space.
    pub fn accept_write(&mut self, native_bytes: usize, clock: &SimulationClock) -> Duration {
        // Transition to writing phase
        if self.phase != BufferPhase::Writing {
            if self.phase == BufferPhase::Idle || self.phase == BufferPhase::Flushing {
                self.write_cycle_count += 1;
            }
            self.phase = BufferPhase::Writing;
            // Reset read cache when switching to write mode
            self.read_cache_bytes = 0;
            self.read_cache_objects = 0;
        }

        self.total_host_bytes += native_bytes as u64;
        self.write_buffered_bytes += native_bytes;
        self.write_buffered_objects += 1;

        // Track high water mark
        if self.write_buffered_bytes > self.high_water_mark_bytes {
            self.high_water_mark_bytes = self.write_buffered_bytes;
        }

        // Update host rate EWMA
        self.update_host_rate(native_bytes, clock);

        // Speed governor
        self.update_speed_governor();

        if clock.speed_factor().is_infinite() {
            return Duration::ZERO;
        }

        // Check for buffer full → stall
        if self.write_buffered_bytes >= self.capacity {
            let excess = self.write_buffered_bytes - self.capacity;
            let drain_rate = self.current_speed.native_rate.max(1);
            let stall_secs = excess as f64 / drain_rate as f64;
            self.stall_time_secs += stall_secs;
            Duration::from_secs_f64(stall_secs)
        } else {
            Duration::ZERO
        }
    }

    /// Record a host read consuming from read cache. Returns a backhitch
    /// duration if the cache was empty, or `Duration::ZERO` normally.
    pub fn record_read(&mut self, native_bytes: usize, clock: &SimulationClock) -> Duration {
        // Transition to reading phase
        if self.phase != BufferPhase::Reading {
            if self.phase == BufferPhase::Idle {
                self.read_cycle_count += 1;
            }
            self.phase = BufferPhase::Reading;
            // Flush any write buffer first
            self.write_buffered_bytes = 0;
            self.write_buffered_objects = 0;
        }

        self.total_host_bytes += native_bytes as u64;

        // Update host rate EWMA
        self.update_host_rate(native_bytes, clock);

        // Speed governor
        self.update_speed_governor();

        if clock.speed_factor().is_infinite() {
            self.read_cache_bytes = self.read_cache_bytes.saturating_sub(native_bytes);
            return Duration::ZERO;
        }

        // Check for cache underrun → backhitch
        if self.read_cache_bytes < native_bytes {
            self.backhitch_count += 1;
            self.read_cache_bytes = 0;
            self.read_cache_objects = 0;
            let backhitch_secs = self.timing.backhitch_sec;
            self.stall_time_secs += backhitch_secs;
            // After backhitch, cache is refilled at tape speed
            self.read_cache_bytes = self.capacity / 2;
            Duration::from_secs_f64(backhitch_secs)
        } else {
            self.read_cache_bytes -= native_bytes;
            if self.read_cache_objects > 0 {
                self.read_cache_objects -= 1;
            }
            Duration::ZERO
        }
    }

    /// Transition to Reading phase; starts filling read cache at tape speed.
    pub fn begin_read_ahead(&mut self) {
        if self.phase != BufferPhase::Reading {
            self.read_cycle_count += 1;
            self.phase = BufferPhase::Reading;
            self.write_buffered_bytes = 0;
            self.write_buffered_objects = 0;
            // Start with some pre-fill
            self.read_cache_bytes = self.capacity / 4;
            self.read_cache_objects = 0;
        }
    }

    /// Mark all write-buffered data as needing immediate commit (flush).
    /// Used on WRITE FILEMARKS (immed=0), unload, etc.
    pub fn flush(&mut self) {
        if self.write_buffered_bytes > 0 {
            self.phase = BufferPhase::Flushing;
        } else {
            self.phase = BufferPhase::Idle;
        }
        self.read_cache_bytes = 0;
        self.read_cache_objects = 0;
    }

    /// Clear all state (on media unload).
    pub fn reset(&mut self) {
        self.write_buffered_bytes = 0;
        self.write_buffered_objects = 0;
        self.read_cache_bytes = 0;
        self.read_cache_objects = 0;
        self.phase = BufferPhase::Idle;
        self.host_rate_ewma = 0.0;
        self.last_io_time = None;
        self.current_speed = self.speed_table.fastest().clone();
    }

    /// Current buffer phase.
    pub fn phase(&self) -> &BufferPhase {
        &self.phase
    }

    /// Capture current state for the dashboard.
    pub fn snapshot(&self) -> BufferSnapshot {
        let cap = self.capacity as f64;
        let total_speed_time: f64 = self.speed_time_secs.iter().sum();

        let speed_time_distribution: Vec<SpeedTimeEntry> = self
            .speed_time_secs
            .iter()
            .enumerate()
            .map(|(i, &time_secs)| {
                let entry = self.speed_table.by_index((i + 1) as u8);
                SpeedTimeEntry {
                    speed_index: (i + 1) as u8,
                    rate_mbps: entry.map(|e| e.native_rate as f64 / 1_000_000.0).unwrap_or(0.0),
                    time_secs,
                    time_pct: if total_speed_time > 0.0 {
                        time_secs / total_speed_time * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect();

        // Compute tape efficiency: useful motion / total motion
        // useful = time tape is moving data; total = useful + backhitch + speed change penalties
        let total_tape_time: f64 = self.speed_time_secs.iter().sum();
        let penalty_time = self.backhitch_count as f64 * self.timing.backhitch_sec
            + self.speed_change_count as f64 * self.timing.speed_change_sec;
        let total_motion = total_tape_time + penalty_time;
        let tape_efficiency_pct = if total_motion > 0.0 {
            Some(total_tape_time / total_motion * 100.0)
        } else {
            None
        };

        BufferSnapshot {
            capacity_bytes: self.capacity,
            write_buffer_bytes: self.write_buffered_bytes,
            write_buffer_pct: if cap > 0.0 {
                self.write_buffered_bytes as f64 / cap * 100.0
            } else {
                0.0
            },
            read_cache_bytes: self.read_cache_bytes,
            read_cache_pct: if cap > 0.0 {
                self.read_cache_bytes as f64 / cap * 100.0
            } else {
                0.0
            },
            objects_in_buffer: self.write_buffered_objects + self.read_cache_objects,
            phase: self.phase.display_name().to_string(),
            current_speed_index: Some(self.current_speed.index),
            current_speed_rate: Some(self.current_speed.native_rate),
            tape_velocity_pct: Some(self.current_speed.velocity_fraction * 100.0),
            host_rate_bytes_sec: if self.host_rate_ewma > 0.0 {
                Some(self.host_rate_ewma as u64)
            } else {
                None
            },
            effective_tape_rate: Some(self.current_speed.native_rate),
            speed_change_count: self.speed_change_count,
            backhitch_count: self.backhitch_count,
            high_water_mark_pct: if cap > 0.0 {
                self.high_water_mark_bytes as f64 / cap * 100.0
            } else {
                0.0
            },
            stall_time_secs: self.stall_time_secs,
            speed_time_distribution,
            tape_efficiency_pct,
            write_cycle_count: self.write_cycle_count,
            read_cycle_count: self.read_cycle_count,
        }
    }

    // ── Private helpers ─────────────────────────────────────────────────

    fn update_host_rate(&mut self, bytes: usize, clock: &SimulationClock) {
        if clock.speed_factor().is_infinite() {
            return;
        }
        let now = Instant::now();
        if let Some(last) = self.last_io_time {
            let dt = now.duration_since(last).as_secs_f64();
            if dt > 0.0 {
                let instant_rate = bytes as f64 / dt;
                if self.host_rate_ewma == 0.0 {
                    self.host_rate_ewma = instant_rate;
                } else {
                    self.host_rate_ewma =
                        self.ewma_alpha * instant_rate + (1.0 - self.ewma_alpha) * self.host_rate_ewma;
                }
            }
        } else {
            // First sample — initialize EWMA
            self.host_rate_ewma = self.current_speed.native_rate as f64;
        }
        self.last_io_time = Some(now);
    }

    fn update_speed_governor(&mut self) {
        // Use EWMA rate (already native) for speed selection.
        // In a real drive, compression ratio would scale this, but since we
        // track native bytes throughout, we use the host rate directly.
        let effective_rate = self.host_rate_ewma as u64;
        let mut new_speed = self.speed_table.select_speed(effective_rate).clone();

        // Buffer-level overrides
        let fill_pct = if self.capacity > 0 {
            match self.phase {
                BufferPhase::Writing | BufferPhase::Flushing => {
                    self.write_buffered_bytes as f64 / self.capacity as f64
                }
                _ => 0.0,
            }
        } else {
            0.0
        };

        match self.phase {
            BufferPhase::Writing | BufferPhase::Flushing => {
                if fill_pct > 0.9 {
                    // Buffer nearly full: force one step faster (drain faster)
                    new_speed = self.speed_table.one_faster(new_speed.index).clone();
                } else if fill_pct < 0.1 && self.write_buffered_bytes > 0 {
                    // Buffer nearly empty: slow down to avoid underrun
                    new_speed = self.speed_table.one_slower(new_speed.index).clone();
                }
            }
            BufferPhase::Reading => {
                let cache_pct = if self.capacity > 0 {
                    self.read_cache_bytes as f64 / self.capacity as f64
                } else {
                    0.0
                };
                if cache_pct < 0.1 && self.read_cache_bytes > 0 {
                    // Cache nearly empty: speed up tape to fill faster
                    new_speed = self.speed_table.one_faster(new_speed.index).clone();
                } else if cache_pct > 0.9 {
                    // Cache nearly full: slow down tape
                    new_speed = self.speed_table.one_slower(new_speed.index).clone();
                }
            }
            _ => {}
        }

        if new_speed.index != self.current_speed.index {
            self.speed_change_count += 1;
            self.current_speed = new_speed;
        }
    }
}

// ── Snapshot types ──────────────────────────────────────────────────────────

/// Snapshot of buffer state for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct BufferSnapshot {
    pub capacity_bytes: usize,
    pub write_buffer_bytes: usize,
    pub write_buffer_pct: f64,
    pub read_cache_bytes: usize,
    pub read_cache_pct: f64,
    pub objects_in_buffer: u32,
    pub phase: String,
    pub current_speed_index: Option<u8>,
    pub current_speed_rate: Option<u64>,
    pub tape_velocity_pct: Option<f64>,
    pub host_rate_bytes_sec: Option<u64>,
    pub effective_tape_rate: Option<u64>,
    pub speed_change_count: u32,
    pub backhitch_count: u32,
    pub high_water_mark_pct: f64,
    pub stall_time_secs: f64,
    pub speed_time_distribution: Vec<SpeedTimeEntry>,
    pub tape_efficiency_pct: Option<f64>,
    pub write_cycle_count: u32,
    pub read_cycle_count: u32,
}

/// Time spent at a specific speed level.
#[derive(Debug, Clone, Serialize)]
pub struct SpeedTimeEntry {
    pub speed_index: u8,
    pub rate_mbps: f64,
    pub time_secs: f64,
    pub time_pct: f64,
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::geometry::LtoGeneration;
    use crate::timing::TimingModel;

    fn test_geometry() -> &'static TapeGeometry {
        LtoGeneration::Lto9.geometry()
    }

    #[test]
    fn speed_table_has_correct_entries() {
        let geo = test_geometry();
        let table = SpeedTable::for_geometry(geo);
        assert_eq!(table.num_speeds(), 14);
        assert_eq!(table.fastest().index, 1);
        assert_eq!(table.slowest().index, 14);
        assert_eq!(table.fastest().native_rate, geo.sustained_rate_bytes_sec);
        // Slowest should be 1/14 of max
        let expected_slowest = geo.sustained_rate_bytes_sec / 14;
        assert_eq!(table.slowest().native_rate, expected_slowest);
    }

    #[test]
    fn speed_table_selects_matching_speed() {
        let geo = test_geometry();
        let table = SpeedTable::for_geometry(geo);
        // Host at full speed → fastest
        let s = table.select_speed(geo.sustained_rate_bytes_sec);
        assert_eq!(s.index, 1);
        // Host at half speed
        let half = geo.sustained_rate_bytes_sec / 2;
        let s = table.select_speed(half);
        assert!(s.native_rate <= half);
        // Host very slow → slowest
        let s = table.select_speed(1000);
        assert_eq!(s.index, 14);
    }

    #[test]
    fn buffer_instant_mode_no_stall() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        // Write more than capacity
        let stall = buf.accept_write(geo.buffer_size_bytes + 1, &clock);
        assert_eq!(stall, Duration::ZERO);

        // Tick clears everything
        buf.tick(&clock);
        assert_eq!(buf.write_buffered_bytes, 0);
    }

    #[test]
    fn buffer_tracks_write_phase() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        assert_eq!(buf.phase, BufferPhase::Idle);
        buf.accept_write(1024, &clock);
        assert_eq!(buf.phase, BufferPhase::Writing);
        assert_eq!(buf.write_cycle_count, 1);
    }

    #[test]
    fn buffer_tracks_read_phase() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let _clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        buf.begin_read_ahead();
        assert_eq!(buf.phase, BufferPhase::Reading);
        assert_eq!(buf.read_cycle_count, 1);
    }

    #[test]
    fn buffer_flush_transitions_to_flushing() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        buf.accept_write(1024, &clock);
        buf.flush();
        assert_eq!(buf.phase, BufferPhase::Flushing);

        // In instant mode, tick clears the flush
        buf.tick(&clock);
        assert_eq!(buf.phase, BufferPhase::Idle);
    }

    #[test]
    fn buffer_snapshot_populated() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        buf.accept_write(512 * 1024, &clock);
        let snap = buf.snapshot();
        assert_eq!(snap.capacity_bytes, geo.buffer_size_bytes);
        assert!(snap.write_buffer_bytes > 0);
        assert_eq!(snap.write_cycle_count, 1);
        assert_eq!(snap.speed_time_distribution.len(), 14);
    }

    #[test]
    fn buffer_reset_clears_state() {
        let geo = test_geometry();
        let timing = TimingModel::default_for_generation(LtoGeneration::Lto9);
        let clock = SimulationClock::instant();
        let mut buf = DriveBuffer::new(geo, timing);

        buf.accept_write(1024, &clock);
        buf.reset();
        assert_eq!(buf.write_buffered_bytes, 0);
        assert_eq!(buf.phase, BufferPhase::Idle);
        assert_eq!(buf.host_rate_ewma, 0.0);
    }
}
