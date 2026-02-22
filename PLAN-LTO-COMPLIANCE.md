# LTO Drive Compliance Plan — Parallel Track Architecture

## Current State Assessment

The SSC crate today implements a **functional but skeletal** tape drive:
- 12 SCSI commands (6-byte forms only), all instant-completion
- Tape modeled as `Vec<TapeRecord>` — a flat list of data blocks and filemarks
- No physical tape geometry (wraps, bands, datasets)
- No buffering simulation (despite MODE SENSE claiming buffered mode)
- No capacity limits, no compression, no write protection
- Minimal sense data — missing INFORMATION field, no deferred errors
- Two VPD pages (00h, 80h), one mode page (static), zero log pages
- No LOCATE, no 16-byte CDBs, no partitions
- DriveSnapshot exposes only: serial, loaded, barcode, position, record_count

The SMC is more complete for its scope but also lacks log pages, diagnostics,
and MODE SELECT support.

---

## Strategy: Foundation Sprint → Parallel Tracks

The old plan was a 10-phase waterfall. The problem: most individual mode pages,
log pages, VPD pages, and commands are *independent of each other* — they only
depend on a shared framework. So we front-load the frameworks, then fan out.

```
                    ┌─── Track A: Commands ──────────────────────┐
                    │                                            │
Foundation ────────├─── Track B: Mode Pages ─────────────────────┤
Sprint             │                                            │
(sequential,       ├─── Track C: Log Pages ──────────────────────┤──→ Integration
 ~1 week)          │                                            │    & Polish
                    ├─── Track D: VPD / INQUIRY ─────────────────┤
                    │                                            │
                    ├─── Track E: Physical Sim (buffer/timing) ──┤
                    │                                            │
                    ├─── Track F: Dashboard ─────────────────────┤
                    │                                            │
                    └─── Track G: SMC Enhancements ──────────────┘
```

All seven tracks can proceed in parallel once the Foundation Sprint lands.
Each track is internally ordered but has no dependencies on other tracks.

---

# FOUNDATION SPRINT

Everything in the sprint is sequential — each piece builds on the previous.
The goal is to land skeleton infrastructure with working stubs so that all
tracks have stable traits and types to code against.

## F1. Tape Media Model & Geometry

Replace `Vec<TapeRecord>` with a model that knows about LTO physics. This is
the single data structure everything else reads from and writes to.

### F1.1 — LTO Generation Constants

```rust
pub enum LtoGeneration { Lto5, Lto6, Lto7, Lto8, Lto9 }

pub struct TapeGeometry {
    pub generation: LtoGeneration,
    pub num_wraps: u32,                 // 280 (LTO-9), 208 (LTO-8), etc.
    pub tracks_per_wrap: u32,           // 32 for LTO-7+
    pub wrap_length_m: f64,             // meters of tape per wrap
    pub total_tape_length_m: f64,       // total tape in cartridge
    pub native_capacity_bytes: u64,     // uncompressed capacity
    pub max_logical_block_bytes: u32,   // 16,777,215 or 8,388,608 (encrypted)
    pub min_logical_block_bytes: u32,   // 1
    pub sustained_rate_bytes_sec: u64,  // native transfer rate
    pub buffer_size_bytes: usize,       // internal buffer size
    pub num_speeds: u8,                 // digital speed matching levels
    pub density_code: u8,               // SCSI density code
}
```

Provide `const` instances for LTO-5 through LTO-9, plus LTO-8 Type M.

### F1.2 — Position Model

```rust
pub struct LogicalPosition {
    pub partition: u8,          // 0-3
    pub block_number: u64,      // logical object identifier (LOI)
    pub file_number: u64,       // count of filemarks from BOP
}

pub struct PhysicalPosition {
    pub wrap: u32,
    pub offset_in_wrap_pct: f64,    // 0.0 = start of wrap, 1.0 = end
}
```

Provide a mapping function `logical_to_physical(logical, geometry) -> physical`
that produces plausible (not bit-accurate) physical positions. This enables
wrap-transition detection, seek-distance estimation, and dashboard display.

### F1.3 — Structured Tape Media

```rust
pub struct TapePartition {
    pub records: Vec<TapeRecord>,
    pub eod_position: u64,
    pub filemark_positions: Vec<u64>,   // fast filemark lookup
    pub bytes_written_native: u64,
    pub bytes_written_compressed: u64,
}

pub struct TapeMedia {
    pub barcode: String,
    pub generation: LtoGeneration,
    pub geometry: &'static TapeGeometry,
    pub partitions: Vec<TapePartition>, // 1-4 partitions
    pub write_protected: bool,
    pub worm: bool,
    pub mam: MamAttributes,
    pub total_loads: u32,
}
```

### F1.4 — MAM Attribute Store (skeleton)

Key-value store of MAM attributes (spec section 6.5). Initially populate with:
- Medium manufacturer, serial, barcode, type
- Load count, bytes written/read
- Medium optimization state

The store is per-cartridge — travels with the media, not the drive. Provide a
`MamAttributes` struct with `get(id) -> Option<Vec<u8>>` and
`set(id, value)` methods. Individual attribute population happens in Track B/C.

## F2. Sense Data Builder

Replace ad-hoc `vec![0x70, ...]` byte arrays with a builder.

```rust
pub struct SenseBuilder {
    format: SenseFormat,        // Fixed(0x70) or Descriptor(0x72)
    sense_key: u8,
    asc: u8,
    ascq: u8,
    information: Option<u64>,   // INFORMATION field (4 or 8 bytes)
    cmd_specific: Option<u64>,  // COMMAND-SPECIFIC INFORMATION
    filemark: bool,
    eom: bool,
    ili: bool,
    valid: bool,
}

impl SenseBuilder {
    pub fn no_sense() -> Self;
    pub fn not_ready(asc: u8, ascq: u8) -> Self;
    pub fn illegal_request(asc: u8, ascq: u8) -> Self;
    pub fn medium_error(asc: u8, ascq: u8) -> Self;
    pub fn blank_check() -> Self;
    pub fn unit_attention(asc: u8, ascq: u8) -> Self;

    pub fn with_information(self, info: u64) -> Self;
    pub fn with_filemark(self) -> Self;
    pub fn with_eom(self) -> Self;
    pub fn with_ili(self) -> Self;

    pub fn build(&self) -> Vec<u8>;     // 18 bytes (fixed) or variable (descriptor)
}
```

Also add a `UnitAttentionQueue` that commands consult before executing. This
is per-I_T nexus. Skeleton implementation: just the queue and the
`check_and_clear()` method; individual UA conditions get added as tracks
produce them.

## F3. Mode Page Framework

Build the engine that MODE SENSE / MODE SELECT dispatch through. Individual
pages are registered as trait objects and filled in by Track B.

```rust
pub trait ModePage: Send + Sync {
    fn page_code(&self) -> u8;
    fn subpage_code(&self) -> u8;            // 0 for no subpage
    fn current_values(&self) -> Vec<u8>;     // page data (without header)
    fn changeable_mask(&self) -> Vec<u8>;    // 1 bits = changeable
    fn default_values(&self) -> Vec<u8>;
    fn saved_values(&self) -> Vec<u8>;
    fn apply_select(&self, data: &[u8]) -> Result<(), SenseBuilder>;
}

pub struct ModePageRegistry {
    pages: Vec<Box<dyn ModePage>>,
}

impl ModePageRegistry {
    pub fn sense(&self, page: u8, subpage: u8, pc: u8) -> Result<Vec<u8>, SenseBuilder>;
    pub fn select(&self, data: &[u8]) -> Result<(), SenseBuilder>;
    pub fn all_pages(&self, pc: u8) -> Vec<u8>;
}
```

Register stub pages that return hardcoded defaults for all mandatory pages.
Track B replaces stubs with real implementations one at a time.

Implement MODE SENSE(6), MODE SENSE(10), MODE SELECT(6), MODE SELECT(10)
command handlers that delegate to this registry.

## F4. Log Page Framework

Same pattern as mode pages. Individual log pages are registered as trait
objects and filled in by Track C.

```rust
pub trait LogPage: Send + Sync {
    fn page_code(&self) -> u8;
    fn subpage_code(&self) -> u8;
    fn parameters(&self) -> Vec<LogParameter>;
    fn reset(&self);    // LOG SELECT reset
}

pub struct LogParameter {
    pub code: u16,
    pub control: u8,        // threshold/cumulative flags
    pub value: Vec<u8>,
}

pub struct LogPageRegistry {
    pages: Vec<Box<dyn LogPage>>,
}
```

Implement LOG SENSE (4Dh) and LOG SELECT (4Ch) command handlers that delegate
to this registry. Register skeleton pages for all mandatory log pages (return
zero counters).

## F5. Enhanced Snapshot & Event Infrastructure

Expand `DriveSnapshot` to its full form now, even though most fields will be
`None` or zero until tracks fill them in. This lets Track F (Dashboard) start
immediately.

```rust
pub struct DriveSnapshot {
    // Identity
    pub serial: String,
    pub generation: LtoGeneration,
    // Media
    pub loaded: bool,
    pub barcode: Option<String>,
    pub write_protected: bool,
    pub worm: bool,
    // Logical position
    pub partition: u8,
    pub block_number: u64,
    pub file_number: u64,
    pub at_bop: bool,
    pub at_eod: bool,
    // Physical position (None if no media)
    pub current_wrap: Option<u32>,
    pub total_wraps: Option<u32>,
    pub position_in_wrap_pct: Option<f64>,
    // Buffer state
    pub write_buffer_pct: f64,
    pub read_cache_pct: f64,
    pub objects_in_buffer: u32,
    pub buffer_state: String,
    // Activity
    pub drive_state: DriveActivity,
    pub tape_speed: Option<u8>,
    pub operation_progress_pct: Option<f64>,
    // Performance
    pub instantaneous_rate_bytes_sec: Option<u64>,
    pub compression_ratio: Option<f64>,
    pub backhitch_count_this_mount: u32,
    // Capacity
    pub capacity_used_pct: Option<f64>,
    pub native_bytes_written: u64,
    pub compressed_bytes_written: u64,
    pub approximate_remaining_mb: Option<u64>,
    // Lifetime
    pub total_loads: u32,
    pub motion_hours: f64,
}

pub enum DriveActivity {
    Empty,
    Loading,
    Unloading,
    Idle,
    Reading,
    Writing,
    Locating,
    Rewinding,
    Calibrating,    // LTO-9 media optimization
    Error,
}
```

Also define the structured WebSocket event types:
```rust
pub enum DriveEvent {
    StateChange { drive: u8, state: DriveActivity },
    PositionUpdate { drive: u8, block: u64, wrap: u32, wrap_pct: f64 },
    BufferUpdate { drive: u8, write_pct: f64, read_pct: f64, state: String },
    Backhitch { drive: u8, from_wrap: u32, to_wrap: u32 },
    Operation { drive: u8, opcode: u8, name: String, duration_us: u64 },
    MediaEvent { drive: u8, event: String, barcode: String },
}
```

Even though events will be sparse initially, the plumbing is in place.

## F6. Time-Scale Configuration

Add a global (or per-drive) time-scale factor from the start:

```rust
pub struct SimulationClock {
    scale: f64,     // 1.0 = realtime, 0.0 = instant, 0.1 = 10x faster
}

impl SimulationClock {
    pub async fn sleep(&self, real_duration: Duration) {
        if self.scale > 0.0 {
            tokio::time::sleep(real_duration.mul_f64(self.scale)).await;
        }
    }
    pub fn scale_duration(&self, d: Duration) -> Duration {
        d.mul_f64(self.scale)
    }
}
```

Passed into drive and changer constructors. CI uses `scale=0.0`. Config file
sets the default. This unblocks Track E (timing) without breaking existing
instant behavior.

## F7. Crate Restructure

The SSC crate is currently a single 706-line `lib.rs`. Split into modules now
to avoid merge conflicts as tracks proceed in parallel:

```
crates/ssc/src/
├── lib.rs                  // Public API, TapeDrive struct, ScsiDevice impl dispatch
├── commands/
│   ├── mod.rs
│   ├── read.rs             // READ(6) handler
│   ├── write.rs            // WRITE(6) handler
│   ├── position.rs         // SPACE, LOCATE, REWIND, READ_POSITION
│   ├── control.rs          // TUR, REQUEST_SENSE, LOAD_UNLOAD, PREVENT_ALLOW
│   ├── inquiry.rs          // INQUIRY, VPD page dispatch
│   ├── mode.rs             // MODE SENSE/SELECT dispatch
│   ├── log.rs              // LOG SENSE/SELECT dispatch
│   ├── filemarks.rs        // WRITE FILEMARKS
│   ├── erase.rs            // ERASE, FORMAT MEDIUM
│   ├── density.rs          // REPORT DENSITY SUPPORT
│   ├── maintenance.rs      // REPORT SUPPORTED OPCODES, TMF, TIMESTAMP
│   └── attribute.rs        // READ/WRITE ATTRIBUTE
├── media/
│   ├── mod.rs
│   ├── geometry.rs         // TapeGeometry, LtoGeneration, constants
│   ├── position.rs         // LogicalPosition, PhysicalPosition, mapping
│   ├── tape.rs             // TapeMedia, TapePartition, TapeRecord
│   └── mam.rs              // MamAttributes
├── mode_pages/
│   ├── mod.rs              // ModePage trait, ModePageRegistry
│   ├── compression.rs      // MP 0Fh
│   ├── device_config.rs    // MP 10h, 10h[01h]
│   ├── partition.rs        // MP 11h
│   ├── control.rs          // MP 0Ah, 0Ah[01h], 0Ah[F0h]
│   ├── error_recovery.rs   // MP 01h
│   ├── disconnect.rs       // MP 02h
│   ├── power.rs            // MP 1Ah
│   ├── exceptions.rs       // MP 1Ch
│   └── medium_config.rs    // MP 1Dh
├── log_pages/
│   ├── mod.rs              // LogPage trait, LogPageRegistry
│   ├── seq_access.rs       // LP 0Ch — capacity / usage
│   ├── error_counters.rs   // LP 02h, 03h, 06h
│   ├── device_status.rs    // LP 11h
│   ├── device_stats.rs     // LP 14h
│   ├── volume_stats.rs     // LP 17h
│   ├── compression.rs      // LP 1Bh
│   ├── tapealerts.rs       // LP 2Eh
│   ├── tape_usage.rs       // LP 30h
│   └── tape_capacity.rs    // LP 31h
├── vpd/
│   ├── mod.rs              // VPD page dispatch
│   ├── supported.rs        // VPD 00h
│   ├── serial.rs           // VPD 80h
│   ├── identification.rs   // VPD 83h
│   ├── firmware.rs         // VPD 03h
│   ├── extended_inquiry.rs // VPD 86h
│   ├── seq_access_cap.rs   // VPD B0h
│   ├── block_protection.rs // VPD B5h
│   └── component_rev.rs    // VPD C0h
├── buffer.rs               // ObjectBuffer, read-ahead, write-gather
├── timing.rs               // TimingModel, SimulationClock
├── sense.rs                // SenseBuilder
├── events.rs               // DriveEvent, event emission
└── snapshot.rs             // DriveSnapshot, DriveActivity
```

Create all files/modules with stub implementations during the sprint. Every
module compiles, every stub returns a reasonable default. This is the key
unlock — after this, any module can be fleshed out independently.

---

# PARALLEL TRACKS

All tracks start after the Foundation Sprint lands. They have no
inter-dependencies except where explicitly noted.

---

## Track A: Command Completeness

Fill in every SCSI command handler, dispatching through the frameworks built in
the Foundation Sprint. Each command is independent.

### A1. Positioning Commands
- LOCATE(10) (2Bh) — seek to block/filemark/EOD, DEST_TYPE, CP bit, IMMED
- LOCATE(16) (92h) — 16-byte form with 8-byte block address
- SPACE(16) (91h) — 16-byte form with 8-byte count
- READ END OF WRAP POSITION (A3h) — wrap boundary info

### A2. Data Commands
- ERASE (19h) — long (full tape) and short (EOD at current position)
- FORMAT MEDIUM (04h) — partition creation, media optimization trigger
- VERIFY (13h) — read-verify, no data transfer

### A3. Reporting Commands
- REPORT DENSITY SUPPORT (44h) — supported media/density table
- REPORT SUPPORTED OPERATION CODES (A3h[0Ch]) — command table with timeouts
- REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS (A3h[0Dh])
- REPORT TIMESTAMP (A3h[0Fh]) / SET TIMESTAMP (A4h[02h])
- READ ATTRIBUTE (8Ch) / WRITE ATTRIBUTE (8Dh) — MAM read/write
- REPORT LUNS enhancements (if needed)

### A4. Write Protection & Overwrite Control
- ALLOW OVERWRITE (82h) — append-only mode position validation
- PREVENT/ALLOW MEDIUM REMOVAL (1Eh)
- SET CAPACITY (0Bh) — Type M cartridge capacity reduction
- Write-protect enforcement across WRITE/ERASE/FORMAT

### A5. Reservation Commands
- PERSISTENT RESERVE IN (5Eh) / OUT (5Fh) — full PR implementation
- RESERVE(6/10) / RELEASE(6/10) — legacy stubs

### A6. Fix Existing Commands
- READ(6): SILI bit, ILI + INFORMATION for under/overlength
- WRITE(6): EOM + early warning detection, capacity enforcement
- SPACE(6): correct residual count in INFORMATION field
- WRITE FILEMARKS: COUNT=0 flush semantics
- LOAD/UNLOAD: full U/M/I/T state machine, HOLD bit, RETEN bit
- READ POSITION: all three service actions (SHORT/LONG/EXTENDED)
- REWIND: deferred error reporting
- REQUEST SENSE: return pending unit attention or deferred sense

---

## Track B: Mode Pages

Each mode page is an independent implementation of the `ModePage` trait,
replacing its stub in the registry.

### B1. Data Compression (MP 0Fh)
- DCE enable/disable (changeable)
- DCC capability (read-only)
- Compression algorithm field
- Sync with MP 10h byte 14

### B2. Device Configuration (MP 10h)
- Write delay time (bytes 6-7, changeable, 30s default in 100ms units)
- SEW (Synchronize at Early Warning)
- Buffer ratios (read/write)
- Active format (read-only)
- SELECT DATA COMPRESSION ALGORITHM (byte 14)
- WTRE (WORM tamper read enable)

### B3. Device Configuration Extension (MP 10h[01h])
- WRITE MODE (overwrite vs append-only, changeable)
- SHORT ERASE MODE
- PEWS (programmable early warning size in MB)
- Encryption policy bits (ACWRE, WRE, etc.)

### B4. Medium Partition (MP 11h)
- Partition count and sizes
- IDP/FDP format descriptor
- Read-only when volume loaded (mostly)

### B5. Control Pages (MP 0Ah, 0Ah[01h], 0Ah[F0h])
- Control: GLTSD, TST, D_SENSE (descriptor vs. fixed sense)
- Control Extension: SCSI-specific flags
- Control Data Protection: LBP enable, method, length

### B6. Read-Write Error Recovery (MP 01h)
- TB (Transfer Block on error)
- PER (Post Error)
- DTE (Disable Transfer on Error)
- Read/write retry counts

### B7. Remaining Pages
- MP 02h: Disconnect-Reconnect (mostly static)
- MP 1Ah: Power Condition (idle/standby timers)
- MP 1Ch: Informational Exceptions (MRIE field, warning control)
- MP 1Dh: Medium Configuration (read-only medium params)
- MP 18h/19h: Protocol-Specific (static for iSCSI)

---

## Track C: Log Pages

Each log page is an independent implementation of the `LogPage` trait,
replacing its stub in the registry. Some pages need hooks into the
command path to update counters — these hooks are part of the page
implementation (e.g., incrementing write error count from the WRITE handler
via a shared counter).

### C1. Sequential Access Device (LP 0Ch) — HIGHEST PRIORITY
The primary capacity page. Parameters:
- 0000h-0003h: Channel/device write/read byte counts
- 0004h-0008h: Approximate capacity values (BOP→EOD, BOP→EW, EW→LEOP,
  BOP→current, buffer capacity)
- 0100h: Cleaning requested
- 8000h-8003h: MB since cleaning, lifetime loads, lifetime cleaning cycles,
  power-on seconds

Updates on: every WRITE, WRITE FILEMARKS, ERASE, LOAD/UNLOAD.

### C2. Error Counters (LP 02h, 03h, 06h)
- Write errors (02h): corrected, uncorrected, total
- Read errors (03h): corrected, uncorrected, total
- Non-medium errors (06h): interface errors

Updates on: READ, WRITE errors.

### C3. Device Statistics (LP 14h)
Lifetime aggregates:
- 0000h: Volume loads
- 0002h: Power-on hours
- 0003h: Medium motion hours
- 000Eh/000Fh: Hard write/read errors
- 0010h-0015h: Duty cycle percentages

Updates on: LOAD/UNLOAD, periodic timer, command execution.

### C4. TapeAlerts (LP 2Eh)
Bitmap of 64 TapeAlert flags. Key flags:
- Cleaning needed, cleaning expired
- Hardware/firmware errors
- Temperature warnings
- Write/read failure predictions

Updates on: error conditions, media events, periodic checks.

### C5. DT Device Status (LP 11h)
- DT DEVICE ACTIVITY field (what drive is doing right now)
- Voltage, temperature, humidity readings (simulated)
- "CALIBRATING" state during LTO-9 media optimization

Updates on: state changes.

### C6. Volume Statistics (LP 17h)
Per-volume (per-mount) metrics:
- Dataset write/read counts
- Bytes written/read this mount
- Write/read retries this mount

Updates on: READ, WRITE, LOAD/UNLOAD.

### C7. Data Compression (LP 1Bh)
Compression effectiveness:
- Bytes from host for writing vs. bytes written to tape
- Bytes read from tape vs. bytes returned to host
- Derives compression ratio

Updates on: READ, WRITE.

### C8. Legacy Pages (LP 30h, 31h)
- Tape Usage (30h): deprecated but still queried. Simple counters.
- Tape Capacity (31h): partition capacity in MiB. Deprecated but queried.

---

## Track D: VPD Pages & INQUIRY

Each VPD page is independent. Also enhance standard INQUIRY.

### D1. Standard INQUIRY Enhancement
- Expand to full 96 bytes
- Product ID: ULT3580-TD{gen} (generation-specific)
- Firmware revision in YMDV format
- Version descriptors: SAM-5 (0x00A0), SPC-4 (0x0460), SSC-4 (0x0560)
- RMB=1, CMDQ=1, proper response format

### D2. VPD 83h — Device Identification (CRITICAL)
T10 vendor ID descriptor, NAA WWNN, relative target port, WWPN.
This is what persistent naming uses.

### D3. VPD 03h — Firmware Designation
LOAD ID, firmware revision, RU name, build date.

### D4. VPD 86h — Extended Inquiry Data
64 bytes. SIMPSUP, protection flags, microcode capabilities.

### D5. VPD B0h — Sequential Access Capabilities
WORM flag. 6 bytes.

### D6. VPD B5h — Logical Block Protection
Supported LBP methods (00h disabled, 01h/02h CRC).

### D7. VPD C0h — Drive Component Revisions
Code name, build time/date, platform string.

### D8. Update VPD 00h
Update supported pages list to include all new pages.

---

## Track E: Physical Simulation (Buffer & Timing)

This track builds the simulation layer that makes the drive *feel* real.
It depends on the media model and position mapping from the Foundation Sprint,
but not on any other track.

### E1. Object Buffer Model
```rust
pub struct ObjectBuffer {
    capacity_bytes: usize,
    write_queue: VecDeque<BufferedBlock>,
    read_cache: VecDeque<CachedBlock>,
    bop_cache: Option<BopCache>,        // LTO-6+
    bytes_in_write_buffer: u64,
    objects_in_write_buffer: u32,
    flush_timer: Option<Instant>,
    state: BufferState,                 // Gathering, Flushing, ReadAhead, Idle
}
```

### E2. Write-Gather Behavior
- WRITE → data enters buffer, report GOOD immediately (buffered mode)
- Buffer accumulates until flush trigger
- Flush triggers: WRITE FILEMARKS(count=0), write delay timer expired,
  buffer full, buffer-to-medium threshold
- Track byte counts for READ POSITION extended form

### E3. Read-Ahead
- After positioning (SPACE, LOCATE, LOAD, REWIND): prime cache
- Subsequent READs served from cache (no simulated tape motion)
- Cache miss → simulated tape start + read delay
- BOP cache (LTO-6+): reserved portion for beginning-of-partition data

### E4. Backhitch Modeling
When read-ahead invalidated or direction reversal needed:
- Estimate delay from `PhysicalPosition` delta
- Same wrap, small reverse: ~1-3s
- Cross-wrap: ~2s per wrap traversed
- Cross-band: ~5-10s
- Emit `DriveEvent::Backhitch` for dashboard

### E5. Timing Model
Per-generation timing constants:
- Sustained transfer rates (native/compressed)
- Rewind time (~80-120s full)
- Load/unload time (~15-20s)
- Locate time (proportional to wrap distance)
- Backhitch time
- Speed change penalty
- LTO-9 media optimization (~1200s)

All durations go through `SimulationClock.sleep()` for time-scaling.

### E6. IMMED Bit Support
For REWIND, LOCATE, LOAD/UNLOAD, WRITE FILEMARKS, ERASE, FORMAT:
- IMMED=0: await simulated delay, then return status
- IMMED=1: return GOOD, spawn background completion task
- Subsequent commands get NOT READY / OPERATION IN PROGRESS (02h/04h/07h)
- TUR polls until complete

### E7. Digital Speed Matching
- Track host transfer rate over sliding window
- Select tape speed to match (avoid shoe-shining)
- Speed change incurs backhitch-like penalty
- Report current speed in DriveSnapshot

---

## Track F: Dashboard & UI

Can start as soon as the Foundation Sprint lands the expanded `DriveSnapshot`
and `DriveEvent` types. Work in this track is front-end (Vue) + back-end
(admin API snapshot enrichment).

### F1. Drive Detail Panel
Replace current simple drive cards with rich detail view:
- Tape position indicator (visual strip: wrap position + direction arrow)
- Buffer gauges (write buffer %, read cache %)
- Activity state with appropriate animation
- Capacity bar (used / remaining / early-warning zone)

### F2. Granular WebSocket Events
Replace `"refresh"` with structured JSON events (DriveEvent enum).
Frontend subscribes and updates reactive state without full snapshot refetch.

### F3. Operation Timeline
Scrolling timeline showing SCSI commands as they execute:
- Command name, duration, result status
- Backhitch markers
- Speed change indicators

### F4. Tape Wrap Visualization
Bird's-eye view of the tape showing:
- Which wraps have been written
- Current head position
- Direction of travel
- Wrap transition animations

### F5. Capacity & Statistics Dashboard
- Per-tape capacity usage chart (from LP 0Ch values)
- Compression ratio history
- Bytes read/written over time
- Drive lifetime statistics

### F6. SMC Visualization Enhancements
- Robot arm position and movement animation
- Slot → drive transfer animation
- Move operation history

---

## Track G: SMC Compliance Enhancement

The media changer work is largely independent of SSC work. It can share the
mode page and log page framework patterns but has its own command set.

### G1. Mode Page Framework for SMC
Same `ModePage` trait, separate registry in SMC crate.
- MP 1Dh: Element Address Assignment (enhance existing)
- MP 1Eh: Transport Geometry
- MP 1Fh: Device Capabilities (what-moves-where matrix)
- MODE SELECT support

### G2. Log Page Framework for SMC
Same `LogPage` trait, separate registry.
- LP 02h/03h: Error counters
- LP 2Eh: TapeAlerts (changer-specific flags)
- LP 14h: Device statistics (move counts, power hours)

### G3. Missing SMC Commands
- MODE SELECT(6/10)
- LOG SENSE / LOG SELECT
- REPORT SUPPORTED OPERATION CODES
- SEND DIAGNOSTIC / RECEIVE DIAGNOSTIC RESULTS

### G4. Changer Timing Simulation
- Robot arm transit time (proportional to slot distance)
- Load/unload into drive (~15-20s, coordinated with SSC)
- IMMED bit on MOVE MEDIUM
- EmitDriveEvent-style events for robot movement

### G5. Changer Sense Data
Improve sense data for changer-specific conditions:
- Destination element full (3Bh/0Eh)
- Source element empty (3Bh/0Dh)
- Medium transport element not ready
- Proper unit attentions on inventory change

---

# INTEGRATION & POLISH

After tracks converge, integration work:

### I1. Cross-Cutting Features
- **Compression simulation**: Mode page (Track B) enables it, write path
  (Track A) applies ratio, log pages (Track C) report it, capacity (C1)
  accounts for it, dashboard (Track F) displays it
- **Append-only mode**: Mode page (B3) enables it, ALLOW OVERWRITE (A4)
  manages it, WRITE (A6) enforces it
- **Write protection**: Media model (F1) flags it, every write command (A)
  checks it, dashboard (F) shows it

### I2. KVM Integration Test Expansion
Expand the existing KVM integration test to validate new functionality:
- `sg_logs` to query log pages
- `sg_modes` to query/set mode pages
- `sg_vpd` to read VPD pages
- `sginfo` for device identification
- `mt` positioning commands (fsf, bsf, eod, seek)
- `mtx` with timing validation

### I3. Data Persistence
- Persist tape media to `.data` files (barcode-named)
- Persist drive lifetime counters to redb
- Persist library inventory to redb
- Persist saved mode pages to redb
- Survive daemon restart without data loss

### I4. Advanced Features (stretch)
- Logical Block Protection (CRC32C)
- Encryption state tracking
- WORM enforcement
- Multi-partition support
- LTO-9 media optimization simulation

---

# Track Dependency Summary

```
Foundation Sprint (F1-F7): strictly sequential, ~1 week
    │
    ├── Track A (Commands):       no deps on other tracks
    ├── Track B (Mode Pages):     no deps on other tracks
    ├── Track C (Log Pages):      no deps on other tracks (counter hooks are internal)
    ├── Track D (VPD/INQUIRY):    no deps on other tracks
    ├── Track E (Physical Sim):   no deps on other tracks
    ├── Track F (Dashboard):      benefits from C1/E1 data but works with stubs
    └── Track G (SMC):            no deps on SSC tracks
```

The only cross-track data flows are counter updates (e.g., WRITE command in
Track A incrementing a byte counter that LP 0Ch in Track C reads). These flow
through shared `Arc<AtomicU64>` counters or `Mutex<DriveState>` fields
established in the Foundation Sprint.

# Ordering Within Tracks

Within each track, items are numbered by priority. Work the highest-impact
items first, but any item within a track can be done at any time.

**Recommended first picks per track (maximum early impact):**
- Track A: A6 (fix existing commands) then A1 (LOCATE — apps need it)
- Track B: B1 (compression) + B2 (device config) — most-queried pages
- Track C: C1 (LP 0Ch capacity) — the single most important log page
- Track D: D1 (standard INQUIRY) + D2 (VPD 83h) — identity
- Track E: E1 (buffer model) → E2 (write-gather) — enables everything else
- Track F: F1 (drive detail panel) — immediate visual payoff
- Track G: G1 (mode page framework) — unlocks the rest
