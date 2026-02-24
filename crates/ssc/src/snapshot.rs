//! Drive snapshot for API/dashboard and drive activity state.

use serde::Serialize;

use crate::buffer::SpeedTimeEntry;
use crate::media::geometry::LtoGeneration;

/// What the drive is currently doing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveActivity {
    /// No cartridge loaded.
    Empty,
    /// Loading a cartridge.
    Loading,
    /// Unloading a cartridge.
    Unloading,
    /// Cartridge loaded, idle.
    Idle,
    /// Reading data from tape.
    Reading,
    /// Writing data to tape.
    Writing,
    /// Seeking to a position (LOCATE/SPACE).
    Locating,
    /// Rewinding tape.
    Rewinding,
    /// LTO-9 media optimization in progress.
    Calibrating,
    /// Error state.
    Error,
}

impl DriveActivity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Loading => "Loading",
            Self::Unloading => "Unloading",
            Self::Idle => "Idle",
            Self::Reading => "Reading",
            Self::Writing => "Writing",
            Self::Locating => "Locating",
            Self::Rewinding => "Rewinding",
            Self::Calibrating => "Calibrating",
            Self::Error => "Error",
        }
    }
}

/// Complete drive state snapshot for API and dashboard consumption.
#[derive(Debug, Clone, Serialize)]
pub struct DriveSnapshot {
    // --- Identity ---
    pub serial: String,
    pub generation: LtoGeneration,

    // --- Media ---
    pub loaded: bool,
    pub barcode: Option<String>,
    pub write_protected: bool,
    pub worm: bool,

    // --- Logical position ---
    pub partition: u8,
    pub block_number: u64,
    pub file_number: u64,
    pub at_bop: bool,
    pub at_eod: bool,

    // --- Physical position (None if no media) ---
    pub current_wrap: Option<u32>,
    pub total_wraps: Option<u32>,
    pub position_in_wrap_pct: Option<f64>,

    // --- Buffer state ---
    pub write_buffer_pct: f64,
    pub read_cache_pct: f64,
    pub objects_in_buffer: u32,
    pub buffer_state: String,

    // --- Activity ---
    pub drive_state: DriveActivity,
    pub tape_speed: Option<u8>,
    pub operation_progress_pct: Option<f64>,

    // --- Performance ---
    pub instantaneous_rate_bytes_sec: Option<u64>,
    pub compression_ratio: Option<f64>,
    pub backhitch_count_this_mount: u32,

    // --- Capacity ---
    pub capacity_used_pct: Option<f64>,
    pub native_bytes_written: u64,
    pub compressed_bytes_written: u64,
    pub approximate_remaining_mb: Option<u64>,

    // --- Lifetime ---
    pub total_loads: u32,
    pub motion_hours: f64,

    // --- Buffer detail ---
    pub buffer_capacity_bytes: usize,
    pub buffer_used_bytes: usize,
    pub read_cache_bytes: usize,
    pub tape_velocity_pct: Option<f64>,
    pub host_rate_bytes_sec: Option<u64>,
    pub tape_rate_bytes_sec: Option<u64>,
    pub speed_change_count: u32,
    pub buffer_backhitch_count: u32,
    pub high_water_mark_pct: f64,
    pub stall_time_secs: f64,
    pub speed_time_distribution: Option<Vec<SpeedTimeEntry>>,
    pub tape_efficiency_pct: Option<f64>,
    pub write_cycle_count: u32,
    pub read_cycle_count: u32,

    // --- Legacy fields for backward compat (used by current admin API) ---
    pub position: usize,
    pub record_count: usize,
}

impl DriveSnapshot {
    /// Create a snapshot for an empty drive (no media loaded).
    pub fn empty(serial: &str, generation: LtoGeneration) -> Self {
        Self {
            serial: serial.to_string(),
            generation,
            loaded: false,
            barcode: None,
            write_protected: false,
            worm: false,
            partition: 0,
            block_number: 0,
            file_number: 0,
            at_bop: false,
            at_eod: false,
            current_wrap: None,
            total_wraps: None,
            position_in_wrap_pct: None,
            write_buffer_pct: 0.0,
            read_cache_pct: 0.0,
            objects_in_buffer: 0,
            buffer_state: "idle".into(),
            drive_state: DriveActivity::Empty,
            tape_speed: None,
            operation_progress_pct: None,
            instantaneous_rate_bytes_sec: None,
            compression_ratio: None,
            backhitch_count_this_mount: 0,
            capacity_used_pct: None,
            native_bytes_written: 0,
            compressed_bytes_written: 0,
            approximate_remaining_mb: None,
            total_loads: 0,
            motion_hours: 0.0,
            buffer_capacity_bytes: 0,
            buffer_used_bytes: 0,
            read_cache_bytes: 0,
            tape_velocity_pct: None,
            host_rate_bytes_sec: None,
            tape_rate_bytes_sec: None,
            speed_change_count: 0,
            buffer_backhitch_count: 0,
            high_water_mark_pct: 0.0,
            stall_time_secs: 0.0,
            speed_time_distribution: None,
            tape_efficiency_pct: None,
            write_cycle_count: 0,
            read_cycle_count: 0,
            position: 0,
            record_count: 0,
        }
    }
}
