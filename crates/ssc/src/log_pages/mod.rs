//! Log page framework — trait, registry, and LOG SENSE/SELECT dispatch.
//!
//! Individual log page implementations register with the registry via the
//! LogPage trait. The framework handles header construction, page code
//! dispatch, and parameter formatting.

use std::sync::{Arc, Mutex};

// ── Shared drive stats ──────────────────────────────────────────────────────

/// Snapshot of drive statistics shared with live log pages.
#[derive(Debug, Clone)]
pub struct DriveStats {
    pub media_loaded: bool,
    pub native_capacity_bytes: u64,
    pub buffer_size_bytes: usize,
    pub total_bytes_written_native: u64,
    pub total_bytes_written_compressed: u64,
    pub total_bytes_read_native: u64,
    pub total_bytes_read_compressed: u64,
    pub partition_count: u8,
    pub partition_native_written: Vec<u64>,
    pub partition_compressed_written: Vec<u64>,
    pub partition_remaining_bytes: Vec<u64>,
    pub total_loads: u32,
    pub compression_enabled: bool,
}

impl Default for DriveStats {
    fn default() -> Self {
        Self {
            media_loaded: false,
            native_capacity_bytes: 0,
            buffer_size_bytes: 0,
            total_bytes_written_native: 0,
            total_bytes_written_compressed: 0,
            total_bytes_read_native: 0,
            total_bytes_read_compressed: 0,
            partition_count: 0,
            partition_native_written: Vec::new(),
            partition_compressed_written: Vec::new(),
            partition_remaining_bytes: Vec::new(),
            total_loads: 0,
            compression_enabled: true,
        }
    }
}

pub type SharedDriveStats = Arc<Mutex<DriveStats>>;

// ── Log parameter wire format ───────────────────────────────────────────────

/// A single log parameter within a log page.
#[derive(Debug, Clone)]
pub struct LogParameter {
    /// Parameter code (16-bit).
    pub code: u16,
    /// Control byte: DU (bit 7), DS (bit 6), TSD (bit 5), ETC (bit 4),
    /// TMC (bits 3-2), FORMAT_AND_LINKING (bits 1-0).
    pub control: u8,
    /// Parameter value (variable length).
    pub value: Vec<u8>,
}

impl LogParameter {
    /// Create a counter parameter (binary format, 4 bytes).
    pub fn counter32(code: u16, value: u32) -> Self {
        Self {
            code,
            control: 0x03, // binary format, list parameter
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create a counter parameter (binary format, 8 bytes).
    pub fn counter64(code: u16, value: u64) -> Self {
        Self {
            code,
            control: 0x03, // binary format, list parameter
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Serialize this parameter into the log page wire format.
    /// Format: code(2) + control(1) + length(1) + value(N).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.value.len());
        bytes.push(((self.code >> 8) & 0xFF) as u8);
        bytes.push((self.code & 0xFF) as u8);
        bytes.push(self.control);
        bytes.push(self.value.len() as u8);
        bytes.extend(&self.value);
        bytes
    }
}

/// Trait for a single log page implementation.
pub trait LogPage: Send + Sync {
    /// Log page code (e.g., 0x0C for Sequential Access Device).
    fn page_code(&self) -> u8;

    /// Subpage code (0 for pages without subpages).
    fn subpage_code(&self) -> u8 {
        0
    }

    /// Return all parameters for this log page.
    fn parameters(&self) -> Vec<LogParameter>;

    /// Reset clearable counters (LOG SELECT).
    fn reset(&self) {}
}

/// Registry of log pages with dispatch methods.
pub struct LogPageRegistry {
    pages: Vec<Box<dyn LogPage>>,
}

impl LogPageRegistry {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Register a log page implementation.
    pub fn register(&mut self, page: Box<dyn LogPage>) {
        self.pages.push(page);
    }

    /// Build LOG SENSE response for a specific page.
    /// Returns the full page (header + parameters), or None if not found.
    pub fn get_page(&self, page_code: u8, subpage: u8) -> Option<Vec<u8>> {
        let page = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)?;

        let params = page.parameters();
        let mut param_bytes = Vec::new();
        for p in &params {
            param_bytes.extend(p.to_bytes());
        }

        let page_length = param_bytes.len() as u16;

        // Log page header: page_code(1) + subpage(1) + page_length(2) + params
        let mut result = Vec::with_capacity(4 + param_bytes.len());
        let byte0 = page.page_code() & 0x3F;
        // If subpage != 0, set SPF bit
        let byte0 = if page.subpage_code() != 0 {
            byte0 | 0x40
        } else {
            byte0
        };
        result.push(byte0);
        result.push(page.subpage_code());
        result.push(((page_length >> 8) & 0xFF) as u8);
        result.push((page_length & 0xFF) as u8);
        result.extend(param_bytes);

        Some(result)
    }

    /// Build the supported log pages page (page 00h).
    pub fn supported_pages(&self) -> Vec<u8> {
        let mut page_codes: Vec<u8> = self.pages.iter().map(|p| p.page_code()).collect();
        page_codes.sort();
        page_codes.dedup();
        // Include page 00h itself
        if !page_codes.contains(&0x00) {
            page_codes.insert(0, 0x00);
        }

        let page_length = page_codes.len() as u16;
        let mut result = Vec::with_capacity(4 + page_codes.len());
        result.push(0x00); // page code
        result.push(0x00); // subpage
        result.push(((page_length >> 8) & 0xFF) as u8);
        result.push((page_length & 0xFF) as u8);
        result.extend(page_codes);
        result
    }

    /// Reset a specific page's counters (LOG SELECT).
    pub fn reset_page(&self, page_code: u8, subpage: u8) -> bool {
        if let Some(page) = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)
        {
            page.reset();
            true
        } else {
            false
        }
    }

    /// Reset all pages (LOG SELECT with page code 0).
    pub fn reset_all(&self) {
        for page in &self.pages {
            page.reset();
        }
    }
}

// --- Stub log pages ---

/// A stub log page that returns fixed/zero parameters.
pub struct StubLogPage {
    code: u8,
    params: Vec<LogParameter>,
}

impl StubLogPage {
    pub fn new(code: u8, params: Vec<LogParameter>) -> Self {
        Self { code, params }
    }

    pub fn empty(code: u8) -> Self {
        Self {
            code,
            params: Vec::new(),
        }
    }
}

impl LogPage for StubLogPage {
    fn page_code(&self) -> u8 {
        self.code
    }

    fn parameters(&self) -> Vec<LogParameter> {
        self.params.clone()
    }
}

// ── LP 0Ch: Sequential Access Device Log Page ───────────────────────────────

struct SequentialAccessLogPage {
    stats: SharedDriveStats,
}

impl LogPage for SequentialAccessLogPage {
    fn page_code(&self) -> u8 {
        0x0C
    }

    fn parameters(&self) -> Vec<LogParameter> {
        let s = self.stats.lock().unwrap();
        let written_mb = s.total_bytes_written_native / 1_000_000;
        let ew_mb = s.native_capacity_bytes * 98 / 100 / 1_000_000;
        let ew_leop_mb = s.native_capacity_bytes * 2 / 100 / 1_000_000;
        let buf_mb = s.buffer_size_bytes as u32 / 1_000_000;

        vec![
            // 0000h: Total channel write bytes (host→drive = native)
            LogParameter::counter64(0x0000, s.total_bytes_written_native),
            // 0001h: Total device write bytes (drive→media = compressed)
            LogParameter::counter64(0x0001, s.total_bytes_written_compressed),
            // 0002h: Total device read bytes (media→drive = compressed)
            LogParameter::counter64(0x0002, s.total_bytes_read_compressed),
            // 0003h: Total channel read bytes (drive→host = native)
            LogParameter::counter64(0x0003, s.total_bytes_read_native),
            // 0004h: Native capacity BOP→EOD (MB)
            LogParameter::counter32(0x0004, written_mb as u32),
            // 0005h: Native capacity BOP→EW (MB)
            LogParameter::counter32(0x0005, ew_mb as u32),
            // 0006h: Min native capacity EW→LEOP (MB)
            LogParameter::counter32(0x0006, ew_leop_mb as u32),
            // 0007h: Native capacity BOP→current (MB) ≈ BOP→EOD
            LogParameter::counter32(0x0007, written_mb as u32),
            // 0008h: Max native capacity in buffer (MB)
            LogParameter::counter32(0x0008, buf_mb),
            // 0100h: Cleaning requested
            LogParameter::counter64(0x0100, 0),
            // 8000h: MB processed since cleaning
            LogParameter::counter32(0x8000, 0),
            // 8001h: Lifetime load cycles
            LogParameter::counter32(0x8001, s.total_loads),
            // 8002h: Lifetime cleaning cycles
            LogParameter::counter32(0x8002, 0),
            // 8003h: Lifetime power-on seconds
            LogParameter::counter32(0x8003, 0),
        ]
    }
}

// ── LP 17h: Volume Statistics Log Page ──────────────────────────────────────

struct VolumeStatisticsLogPage {
    stats: SharedDriveStats,
}

impl LogPage for VolumeStatisticsLogPage {
    fn page_code(&self) -> u8 {
        0x17
    }

    fn parameters(&self) -> Vec<LogParameter> {
        let s = self.stats.lock().unwrap();
        let valid = if s.media_loaded { 0x01u64 } else { 0x00 };
        let cap_mb = s.native_capacity_bytes / 1_000_000;
        let used_mb = s.total_bytes_written_native / 1_000_000;

        let mut params = vec![
            // 0000h: Page valid
            LogParameter::counter64(0x0000, valid),
            // 0016h: Total native capacity MB
            LogParameter::counter64(0x0016, cap_mb),
            // 0017h: Used native capacity MB
            LogParameter::counter64(0x0017, used_mb),
            // 0018h: Application design capacity
            LogParameter::counter64(0x0018, cap_mb),
        ];

        // Per-partition stats (simplified — partition 0 only for now)
        for i in 0..s.partition_count as u16 {
            let native_written = s.partition_native_written.get(i as usize).copied().unwrap_or(0);
            let remaining = s.partition_remaining_bytes.get(i as usize).copied().unwrap_or(0);
            // 0202h+i: Per-partition native capacity written
            params.push(LogParameter::counter64(0x0202 + i, native_written / 1_000_000));
            // 0204h+i: Per-partition remaining to EW
            params.push(LogParameter::counter64(0x0204 + i * 2, remaining * 98 / 100 / 1_000_000));
        }

        params
    }
}

// ── LP 1Bh: Data Compression Log Page ───────────────────────────────────────

struct DataCompressionLogPage {
    stats: SharedDriveStats,
}

impl LogPage for DataCompressionLogPage {
    fn page_code(&self) -> u8 {
        0x1B
    }

    fn parameters(&self) -> Vec<LogParameter> {
        let s = self.stats.lock().unwrap();

        // Read compression ratio × 100
        let read_ratio_x100 = if s.total_bytes_read_compressed > 0 {
            s.total_bytes_read_native * 100 / s.total_bytes_read_compressed
        } else {
            0
        };

        // Write compression ratio × 100
        let write_ratio_x100 = if s.total_bytes_written_compressed > 0 {
            s.total_bytes_written_native * 100 / s.total_bytes_written_compressed
        } else {
            0
        };

        vec![
            LogParameter::counter64(0x0000, read_ratio_x100),
            LogParameter::counter64(0x0001, write_ratio_x100),
            LogParameter::counter64(0x0100, if s.compression_enabled { 1 } else { 0 }),
        ]
    }
}

// ── LP 31h: Tape Capacity Log Page (Legacy) ─────────────────────────────────

struct TapeCapacityLogPage {
    stats: SharedDriveStats,
}

impl LogPage for TapeCapacityLogPage {
    fn page_code(&self) -> u8 {
        0x31
    }

    fn parameters(&self) -> Vec<LogParameter> {
        let s = self.stats.lock().unwrap();
        let cap_mib = s.native_capacity_bytes / (1024 * 1024);

        let mut params = Vec::new();
        // For each partition: remaining capacity MiB, then max capacity MiB
        for i in 0..std::cmp::max(s.partition_count, 1) as u16 {
            let remaining = s.partition_remaining_bytes.get(i as usize).copied().unwrap_or(0);
            let remaining_mib = remaining / (1024 * 1024);
            // 0001h, 0002h: partition remaining capacity MiB
            params.push(LogParameter::counter32(0x0001 + i, remaining_mib as u32));
        }
        // Pad to 2 partitions
        if s.partition_count < 2 {
            params.push(LogParameter::counter32(0x0002, 0));
        }
        for i in 0..std::cmp::max(s.partition_count, 1) as u16 {
            // 0003h, 0004h: partition max capacity MiB
            params.push(LogParameter::counter32(0x0003 + i, cap_mib as u32));
        }
        if s.partition_count < 2 {
            params.push(LogParameter::counter32(0x0004, 0));
        }

        params
    }
}

// ── Default registry ────────────────────────────────────────────────────────

/// Create the default log page registry with live implementations for
/// capacity/compression pages and stubs for error/alert pages.
pub fn default_registry(stats: SharedDriveStats) -> LogPageRegistry {
    let mut reg = LogPageRegistry::new();

    // LP 02h: Write Error Counters
    reg.register(Box::new(StubLogPage::new(
        0x02,
        vec![
            LogParameter::counter32(0x0000, 0),
            LogParameter::counter32(0x0001, 0),
            LogParameter::counter32(0x0002, 0),
            LogParameter::counter32(0x0003, 0),
            LogParameter::counter32(0x0005, 0),
            LogParameter::counter64(0x0006, 0),
        ],
    )));

    // LP 03h: Read Error Counters
    reg.register(Box::new(StubLogPage::new(
        0x03,
        vec![
            LogParameter::counter32(0x0000, 0),
            LogParameter::counter32(0x0001, 0),
            LogParameter::counter32(0x0002, 0),
            LogParameter::counter32(0x0003, 0),
            LogParameter::counter32(0x0005, 0),
            LogParameter::counter64(0x0006, 0),
        ],
    )));

    // LP 06h: Non-Medium Errors
    reg.register(Box::new(StubLogPage::new(
        0x06,
        vec![LogParameter::counter32(0x0000, 0)],
    )));

    // LP 0Ch: Sequential Access Device — live
    reg.register(Box::new(SequentialAccessLogPage {
        stats: stats.clone(),
    }));

    // LP 11h: DT Device Status
    reg.register(Box::new(StubLogPage::empty(0x11)));

    // LP 14h: Device Statistics
    reg.register(Box::new(StubLogPage::new(
        0x14,
        vec![
            LogParameter::counter32(0x0000, 0),
            LogParameter::counter32(0x0001, 0),
            LogParameter::counter32(0x0002, 0),
            LogParameter::counter32(0x0003, 0),
        ],
    )));

    // LP 17h: Volume Statistics — live
    reg.register(Box::new(VolumeStatisticsLogPage {
        stats: stats.clone(),
    }));

    // LP 1Bh: Data Compression — live
    reg.register(Box::new(DataCompressionLogPage {
        stats: stats.clone(),
    }));

    // LP 2Eh: TapeAlerts
    reg.register(Box::new(StubLogPage::empty(0x2E)));

    // LP 30h: Tape Usage (legacy)
    reg.register(Box::new(StubLogPage::empty(0x30)));

    // LP 31h: Tape Capacity — live
    reg.register(Box::new(TapeCapacityLogPage { stats }));

    reg
}
