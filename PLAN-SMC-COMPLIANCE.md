# SMC Compliance Plan — Quantum Scalar Library Simulation

## Current State Assessment

The SMC crate (`crates/smc/src/lib.rs`) is a single 703-line file implementing a
functional but minimal media changer. Key gaps:

### Critical Issues

1. **Element addressing is wrong.** The spec mandates fixed starting addresses:
   - MTE at `0001h`, I/E at `0010h`, DTE at `0100h`, STE at `1000h`
   - Our implementation uses sequential: MTE=0, DTE=1..N, STE=N+1..N+S, IEE=N+S+1
   - This means `mtx` works by accident (it reads the 1Dh mode page), but any
     hardcoded address assumptions will be wrong.

2. **INQUIRY version descriptors are outdated.** Reports SAM-2/SAM-4/SPC-3/SMC-2
   instead of SAM-5 (00A0h)/SPC-4 (0460h)/SMC-3 (0480h).

3. **No Unit Attention management.** The spec requires a FIFO queue of UA conditions
   per I_T nexus for power-on, door open/close, element status changes, firmware
   updates, reservation changes, and mode parameter changes.

4. **No PREVENT/ALLOW enforcement.** The command is accepted but doesn't actually
   prevent MOVE MEDIUM to I/E elements.

### Missing Commands (15 of 24 spec commands unimplemented)

| Command | Opcode | Priority | Notes |
|---------|--------|----------|-------|
| MODE SELECT(6) | 15h | High | Spec says no changeable params; accept gracefully |
| MODE SELECT(10) | 55h | High | Same |
| REPORT LUNS | A0h | High | Required for LUN discovery |
| LOG SENSE | 4Dh | High | Required for TapeAlert monitoring |
| INIT ELEMENT STATUS WITH RANGE | E7h | Medium | Ranged inventory scan |
| POSITION TO ELEMENT | 2Bh | Medium | Pre-position robotics |
| RESERVE ELEMENT(6/10) | 16h/56h | Medium | Whole-LU reservation |
| RELEASE ELEMENT(6/10) | 17h/57h | Medium | Release reservation |
| SEND DIAGNOSTIC | 1Dh | Low | Self-test only |
| READ BUFFER | 3Ch | Low | Diagnostic validation |
| WRITE BUFFER | 3Bh | Low | Echo buffer / firmware |
| EXCHANGE MEDIUM | A6h | Low | i7 RAPTOR only |
| REPORT ELEMENT INFO | 9Eh | Low | i7 RAPTOR / i3/i6 only |
| SEND VOLUME TAG | B6h | Low | i2000/i6000 only |
| REQUEST VOL ELEM ADDR | B5h | Low | i2000/i6000 only |

### Missing Mode Pages (6 of 7 spec pages unimplemented)

Only **1Dh** (Element Address Assignment) exists. Missing:
- **1Ch** — Informational Exceptions Control (TapeAlert polling)
- **1Eh** — Transport Geometry Parameters (rotate, member number)
- **1Fh/00h** — Device Capabilities (source→dest movement matrix)
- **1Fh/41h** — Extended Device Capabilities (MVPRV, magazine flags, I/E control)
- **02h** — Disconnect-Reconnect (FC-specific, stub OK)
- **18h/19h** — FC LU/Port Control (FC-specific, stub OK)

### Missing Log Pages (all 5 unimplemented)

- **00h** — Supported Log Pages
- **0Dh** — Temperature
- **12h** — TapeAlert Response (bitmap format)
- **2Eh** — TapeAlert (64 individual flag parameters)
- **30h** — Humidity

### Missing VPD Pages

Only 00h, 80h, 83h exist. Missing:
- **85h** — Management Network Addresses
- **C8h** — Vendor-Specific Device Capabilities (failover flags)

### Incomplete READ ELEMENT STATUS

Missing ~25 descriptor fields across element types:
- **All types**: Except flag, ASC/ASCQ, ED (Element Disabled), Medium Type
- **I/E**: OIR, CMC, InEnab, ExEnab, ImpExp flags
- **DTE**: DVCID support (device identification, up to 64 bytes)
- **STE**: Access flag (properly)

### No Timing / Physical Simulation

- MOVE MEDIUM completes instantly — no robot travel time
- No state machine (Ready / Not Ready / Offline)
- No simulation of picker motion, door/magazine state
- No backhitch-equivalent metrics (pick retries, scan retries)

---

## Strategy: Foundation Sprint → Parallel Tracks

Same approach as the LTO drive plan. Front-load shared infrastructure, then
individual features can be implemented independently.

```
                    ┌─── Track A: Commands ──────────────────────┐
                    │                                            │
Foundation ────────├─── Track B: Mode Pages ─────────────────────┤
Sprint             │                                            │
(sequential)       ├─── Track C: Log Pages ──────────────────────┤──→ Integration
                    │                                            │    & Polish
                    ├─── Track D: VPD / INQUIRY ─────────────────┤
                    │                                            │
                    ├─── Track E: Element Status Enhancement ────┤
                    │                                            │
                    └─── Track F: Physical Sim (timing/state) ──┘
```

---

# FOUNDATION SPRINT

## F1: Fix Element Addressing (CRITICAL)

The spec mandates well-known starting addresses. All Quantum Scalar models use:

| Element Type | Start Address | Notes |
|-------------|---------------|-------|
| MTE | `0x0001` | Always 1 picker |
| I/E | `0x0010` | Mailslot(s) |
| DTE | `0x0100` | Drives |
| STE | `0x1000` | Storage slots |

Current code uses sequential 0-based addresses. Fix:
- Change `start_picker=0x0001`, `start_iee=0x0010`, `start_drive=0x0100`,
  `start_slot=0x1000`
- Update `elements` to be a `BTreeMap<u16, Element>` instead of `Vec<Element>`
  (sparse address space doesn't suit contiguous indexing)
- Update all address lookups in MOVE MEDIUM, READ ELEMENT STATUS, MODE SENSE 1Dh
- Update drive index calculation: `drive_idx = addr - start_drive`

## F2: Crate Restructure

Break the 703-line monolith into a module tree mirroring the SSC crate:

```
crates/smc/src/
  lib.rs              — MediaChanger struct, ScsiDevice dispatch, snapshot
  state.rs            — ChangerState, Element, ElementMap, UA queue
  sense.rs            — SenseBuilder (or re-export from ssc)
  commands/
    mod.rs            — Opcode constants
    inquiry.rs        — INQUIRY + VPD dispatch
    mode.rs           — MODE SENSE/SELECT
    log.rs            — LOG SENSE
    move_medium.rs    — MOVE MEDIUM
    element_status.rs — READ ELEMENT STATUS
    control.rs        — TUR, REQUEST SENSE, PREVENT/ALLOW, INIT ELEM STATUS
    position.rs       — POSITION TO ELEMENT
    reservation.rs    — RESERVE/RELEASE ELEMENT
    diagnostic.rs     — SEND DIAGNOSTIC, READ/WRITE BUFFER
    report_luns.rs    — REPORT LUNS
  mode_pages/
    mod.rs            — Registry (reuse SSC pattern or share trait)
  log_pages/
    mod.rs            — Registry
  vpd/
    mod.rs            — VPD page builders
  timing.rs           — Robot timing model, SimulationClock integration
  snapshot.rs         — ChangerSnapshot, ElementSnapshot (enhanced)
  events.rs           — ChangerEvent types for WebSocket
```

## F3: Sense Data Builder

Either share the SSC `SenseBuilder` (extract to a common crate or re-export) or
create an SMC-specific one. The Quantum spec defines sense data with:
- INFORMATION field carrying element addresses (bytes 3-6)
- SKSV + field pointer for ILLEGAL REQUEST
- Full ASC/ASCQ table (~60 codes, see spec file 22)

Named constructors needed:
- `source_element_empty()` — 05h / 3Bh,0Eh
- `destination_element_full()` — 05h / 3Bh,0Dh
- `element_disabled()` — 05h / 3Bh,18h
- `medium_removal_prevented()` — 05h / 53h,02h
- `drive_not_installed()` — 04h / 83h,04h
- `magazine_not_installed()` — 05h / 3Bh,12h
- `magazine_not_accessible()` — 05h / 3Bh,11h
- `becoming_ready()` — 02h / 04h,01h
- `not_ready_manual_intervention()` — 02h / 04h,03h
- `power_on_reset()` — 06h / 29h,00h
- `element_status_changed()` — 06h / 28h,00h
- `mode_parameters_changed()` — 06h / 2Ah,01h
- `incompatible_medium()` — 05h / 30h,00h
- Plus the standard ones: `invalid_opcode()`, `invalid_field_in_cdb()`, etc.

## F4: State Machine & Unit Attention Queue

Add a proper library state model:

```rust
enum LibraryState {
    Initializing,     // After power-on, scanning inventory
    Ready,            // Normal operation
    NotReady(String), // Offline, door open, etc.
}
```

Add per-initiator Unit Attention queue:
```rust
struct UnitAttention {
    sense_key: u8,
    asc: u8,
    ascq: u8,
}

// In ChangerState:
ua_queue: Vec<UnitAttention>,  // FIFO, cleared after delivery
```

TUR and every command should check for pending UAs before executing.

## F5: Mode Page & Log Page Frameworks

Reuse the SSC pattern (or extract to a shared crate):
- `ModePage` trait with `page_code()`, `subpage_code()`, `page_data(pc)`, `apply_select()`
- `ModePageRegistry` with `get_page()`, `get_all_pages()`, `apply_select()`
- `LogPage` trait with `page_code()`, `parameters()`, `reset()`
- `LogPageRegistry` with `get_page()`, `supported_pages()`

Register stub pages for all spec-required pages.

## F6: Enhanced Snapshot & Events

Expand `ChangerSnapshot` for dashboard:

```rust
pub struct ChangerSnapshot {
    // Existing
    pub vendor: String,
    pub product: String,
    pub serial: String,
    pub num_drives: u16,
    pub num_slots: u16,
    pub num_import_export: u16,
    pub elements: Vec<ElementSnapshot>,
    // New
    pub state: LibraryState,
    pub temperature_c: Option<u8>,
    pub humidity_pct: Option<u8>,
    pub total_moves: u64,
    pub picker_position: Option<u16>,  // Current element address picker is near
    pub active_alerts: Vec<u16>,       // Active TapeAlert flag numbers
    pub prevent_medium_removal: bool,
    pub firmware_version: String,
}

pub struct ElementSnapshot {
    // Existing
    pub address: u16,
    pub element_type: ElementType,
    pub full: bool,
    pub barcode: Option<String>,
    pub source_element: u16,
    // New
    pub access: bool,
    pub except: bool,
    pub disabled: bool,
    pub asc_ascq: Option<(u8, u8)>,
    pub medium_type: MediumType,
    pub import_export: bool,      // I/E: placed by operator
    pub operator_intervention: bool, // I/E: OIR flag
}
```

Add `ChangerEvent` for WebSocket:
- `MoveStarted { source, dest, barcode }`
- `MoveCompleted { source, dest, barcode, duration_ms }`
- `MoveFailed { source, dest, reason }`
- `InventoryScanStarted`
- `InventoryScanCompleted { elements_scanned }`
- `AlertRaised { flag, severity, description }`
- `StateChanged { old, new }`

## F7: Timing Model

Robot motion timing for simulation:

```rust
pub struct RobotTimingModel {
    /// Time to pick a cartridge from a slot (seconds).
    pub pick_sec: f64,
    /// Time to place a cartridge into a slot (seconds).
    pub place_sec: f64,
    /// Time to load a cartridge into a drive (seconds).
    pub drive_load_sec: f64,
    /// Time to unload a cartridge from a drive (seconds).
    pub drive_unload_sec: f64,
    /// Horizontal travel speed (slots per second).
    pub travel_speed_slots_per_sec: f64,
    /// Vertical travel speed (rows per second).
    pub travel_speed_rows_per_sec: f64,
    /// I/E door open/close time (seconds).
    pub door_cycle_sec: f64,
    /// Full inventory scan time per slot (seconds).
    pub scan_per_slot_sec: f64,
}
```

Integrate with `SimulationClock` from SSC crate for time scaling.

---

# PARALLEL TRACKS

After the foundation sprint, these tracks are independent.

---

## Track A: Command Completeness

### A1: MODE SELECT(6/10)

The spec says the library has **no changeable parameters**. Implementation:
- Accept the command, parse the header
- If parameter list contains pages, validate format but ignore values
- If SP (Save Pages) bit is set, return `SAVING PARAMETERS NOT SUPPORTED` (39h/00h)
- Return GOOD status

### A2: REPORT LUNS

Simple — return a single LUN (the media changer itself):
- 4-byte LUN list length (0008h = 8 bytes for 1 LUN)
- 4 bytes reserved
- 8-byte LUN descriptor (byte 1 = LUN number, rest 00h)

### A3: POSITION TO ELEMENT

Move picker without moving media:
- Parse CDB bytes 2-3 (transport addr) and 4-5 (destination addr)
- Validate destination exists and is not MTE type
- Update `picker_position` in state
- Apply timing model for travel distance
- Return GOOD

### A4: RESERVE/RELEASE ELEMENT

Whole-LU reservation only (element reservations not supported):
- **RESERVE**: Record reserving I_T nexus; reject other initiators' commands
  with RESERVATION CONFLICT (status 18h)
- **RELEASE**: Clear reservation if from same I_T nexus
- Both 6-byte and 10-byte variants; all unsupported fields must be 0

### A5: INIT ELEMENT STATUS WITH RANGE (E7h)

Enhanced version of 07h:
- Parse Range bit, Starting Element Address, Number of Elements
- Parse Fast/NBL bits (both mean "media presence only" — for us, same behavior)
- If Range=0: re-scan all elements (same as 07h)
- If Range=1: re-scan specified range
- For simulation: update barcodes/full flags from state, return GOOD

### A6: SEND DIAGNOSTIC

Self-test only:
- If SelfTest bit set: simulate a self-test, return GOOD
- If PF=1 or parameter list length > 0: return INVALID FIELD IN CDB
- If DevOfl=1 or UnitOfl=1: return INVALID FIELD IN CDB

### A7: READ/WRITE BUFFER

Diagnostic echo buffer:
- **READ BUFFER Descriptor (mode 3)**: return 4-byte descriptor with buffer capacity
- **READ BUFFER Data (mode 2)**: return data from internal buffer
- **READ BUFFER Echo (mode 0Ah)**: return last WRITE BUFFER Echo data
- **WRITE BUFFER Data (mode 2)**: store data in internal buffer
- **WRITE BUFFER Echo (mode 0Ah)**: store echo data for retrieval

---

## Track B: Mode Pages

### B1: Device Capabilities (1Fh)

The most important missing page — defines the movement matrix:

```
StorDT=1  StorIE=1  StorST=1  StorMT=0  ACE=0/1  VTRP=1  S2C=1
```

Movement matrix (which source→dest combinations are valid):
- MTE→STE: yes, MTE→IEE: yes, MTE→DTE: yes
- STE→STE: yes, STE→IEE: yes, STE→DTE: yes
- IEE→STE: yes, IEE→IEE: yes, IEE→DTE: yes
- DTE→STE: yes, DTE→IEE: yes, DTE→DTE: yes
- Nothing→MTE as destination

MOVE MEDIUM should validate against this matrix.

### B2: Transport Geometry (1Eh)

Simple 2-byte page:
- Rotate=0 (no double-sided media)
- Member Number=0 (single transport element)

### B3: Informational Exceptions Control (1Ch)

10-byte page controlling TapeAlert reporting:
- Dexcpt=1 (disable exceptions — must poll via LOG SENSE)
- MRIE=0 (no asynchronous reporting)
- Interval Timer and Report Count = 0

### B4: Extended Device Capabilities (1Fh/41h)

Subpage format with flags for I/E control, magazine detection, exchange capability:
- MVPRV=1 (prevent moves to I/E when medium removal prevented)
- USRCL=1, USROP=1 (operator controls I/E open/close)
- IEMGZ=1, SMGZ=1 (magazine-based I/E and storage)
- LCKIE=1 (PREVENT/ALLOW locks I/E)
- TREXC=1 (true exchange capable — for i7 RAPTOR)

### B5: FC Pages (02h, 18h, 19h) — Stubs

Return reasonable defaults; we're iSCSI-only so these are compatibility stubs.

---

## Track C: Log Pages

### C1: Supported Log Pages (00h)

Return list: 00h, 0Dh, 12h, 2Eh, 30h

### C2: Temperature (0Dh)

Single parameter (code 0000h), 2 bytes:
- Report a simulated temperature (e.g., 22°C default)
- Dashboard can override for testing

### C3: Humidity (30h)

Single parameter (code 0000h), 2 bytes:
- Report simulated humidity (e.g., 45% default)

### C4: TapeAlert (2Eh)

64 individual parameters (codes 0001h-0040h), each 1 byte:
- Default all to 0 (inactive)
- Support setting flags programmatically for testing:
  - Flag 1: Drive Communication Failure (C)
  - Flag 13: Library Pick Retry (W)
  - Flag 14: Library Place Retry (W)
  - Flag 16: Library Door (C)
  - Flag 17: Mailbox Mechanical Problem (C)
  - Flag 23: Library Scan Retry (W)

### C5: TapeAlert Response (12h)

Alternative bitmap format — single 8-byte parameter encoding all 64 flags.

---

## Track D: VPD & INQUIRY Enhancement

### D1: Fix Version Descriptors

Update INQUIRY response:
- SAM-5: `0x00A0`
- SPC-4: `0x0460`
- SMC-3: `0x0480`

### D2: VPD 85h — Management Network Addresses

Return HTTP/HTTPS URLs for the management interface:
- Service Type 02h (Diagnostics Web): `https://host:port/`
- Service Type 03h (Management): `https://host:port/api/`

### D3: VPD C8h — Device Capabilities

4-byte vendor-specific page:
- ADVFO=0 (no advanced failover)
- BASICFO=0 (no basic failover)

### D4: Enhance VPD 83h

Add NAA descriptor for proper WWNN/WWPN device identification
(consistent with how we did it in SSC).

### D5: INQUIRY Response Polish

- Set RMB=1 (removable medium)
- Set BarC=1 in byte 6 (barcode scanner installed)
- Proper additional length calculation
- Full firmware revision string in bytes 36-54

---

## Track E: Element Status Enhancement

### E1: Full Descriptor Fields

Add all missing flags to element descriptors:

**All types**: Except, ASC/ASCQ, ED (Element Disabled), Medium Type

**I/E elements**:
- OIR (Operator Intervention Required)
- CMC=0 (not connected to another changer)
- InEnab=1 (import enabled)
- ExEnab=1 (export enabled)
- ImpExp (placed by operator vs robot)
- Access=1

**DTE elements**:
- Access (1 = cartridge accessible, i.e. drive is ready)
- DVCID support: when CDB DVCID bit=1, append 64-byte device identification
  descriptor to DTE descriptors (from drive's INQUIRY VPD 83h data)

**STE elements**:
- Access=1 (always accessible unless magazine removed)

### E2: Medium Type Classification

Track media type per cartridge:
- 000b = Data (default)
- 001b = Cleaning
- 011b = Diagnostic
- 100b = WORM
- 101b = Microcode

Derive from barcode suffix conventions (CLNxxxL9 = cleaning, etc.).

### E3: Exception Reporting

When drives are removed/offline, report:
- Except=1
- ASC/ASCQ = 83h/04h (not installed) or 3Bh/1Ah (removed)
- ED=1 (element disabled)

When magazines are not installed:
- Except=1
- ASC/ASCQ = 3Bh/12h (not installed)
- ED=1

---

## Track F: Physical Simulation

### F1: Robot Motion Timing

MOVE MEDIUM should simulate realistic timing:
1. Calculate travel distance: `|source_addr - dest_addr|` mapped to physical distance
2. Pick time (1-3 sec) + Travel time + Place time (1-3 sec)
3. Drive load/unload adds extra time (15-30 sec)
4. Total scaled by SimulationClock

During the move:
- Library state = "Moving" with progress info
- Emit WebSocket events: MoveStarted, MoveCompleted

### F2: Inventory Scan Simulation

INITIALIZE ELEMENT STATUS should simulate scanning:
- Time proportional to number of elements × scan_per_slot_sec
- Library state = "Scanning" during operation
- Accept command immediately (return GOOD) but set NOT READY until complete
- TUR returns NOT READY / BECOMING READY during scan

### F3: PREVENT/ALLOW Enforcement

Actually enforce medium removal prevention:
- Track per-initiator prevent state
- MOVE MEDIUM to I/E: check prevent flag, return 53h/02h if prevented
- Clear on: Allow command, I_T nexus loss, reset

### F4: Move Validation Enhancement

Validate moves against Device Capabilities page:
- Check source→dest combination is in the movement matrix
- Check media compatibility (cleaning tape → drive OK, cleaning tape → slot OK)
- On failure, attempt alternate placement (return UA 06h/28h,00h on success)

### F5: Picker Position Tracking

Track where the picker currently is:
- Update after every MOVE MEDIUM and POSITION TO ELEMENT
- Include in snapshot for dashboard visualization
- Use for travel time calculation

---

# INTEGRATION & POLISH

After all tracks complete:

1. **Cross-track**: MOVE MEDIUM uses Device Capabilities (Track B) for validation,
   timing model (Track F) for duration, enhanced sense (Foundation) for errors,
   and element status (Track E) for proper descriptor updates.

2. **Dashboard**: Changer visualization with picker animation, slot grid,
   drive status, alert panel, temperature/humidity gauges.

3. **KVM Testing**: Verify `mtx status`, `mtx load`, `mtx unload`, `mtx transfer`
   all work correctly with the enhanced implementation.

4. **Shared Infrastructure**: Consider extracting common SCSI types (SenseBuilder,
   ModePage trait, LogPage trait) into a shared crate used by both SSC and SMC.

5. **Persistence**: Save library state to redb so element status survives restarts.
