//! SMC (SCSI Media Changer) — Quantum Scalar tape library emulation.
//!
//! This crate emulates a Quantum Scalar tape library robot. It implements
//! the ScsiDevice trait from the iscsi-target crate to handle SCSI CDB
//! dispatch for media changer commands (MOVE MEDIUM, READ ELEMENT STATUS,
//! INQUIRY, MODE SENSE, LOG SENSE, etc.).

pub mod commands;
pub mod events;
pub mod log_pages;
pub mod mode_pages;
pub mod sense;
pub mod snapshot;
pub mod state;
pub mod timing;
pub mod vpd;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use iscsi_target::{MediaLoadNotify, ScsiDevice, ScsiResult, SimulationClock};
use tokio::sync::broadcast;
use tracing::trace;

use commands::opcodes::*;
use log_pages::LogPageRegistry;
use mode_pages::ModePageRegistry;
use state::ChangerState;
use timing::RobotTimingModel;

// Re-export types used by vtld/admin
pub use snapshot::{ChangerSnapshot, ElementSnapshot, ElementType};
pub use state::LibraryState;

/// A SCSI Media Changer device emulating a Quantum Scalar library.
pub struct MediaChanger {
    /// Standard INQUIRY response data (96 bytes).
    inquiry_data: Vec<u8>,
    /// Library serial number.
    serial: String,
    /// Vendor string (space-padded to 8 bytes).
    vendor: String,
    /// Product string (space-padded to 16 bytes).
    product: String,
    /// Protected internal state.
    state: Mutex<ChangerState>,
    /// Drive notification handles (indexed by drive number).
    drives: Vec<Arc<dyn MediaLoadNotify>>,
    /// Mode page registry.
    mode_pages: ModePageRegistry,
    /// Log page registry.
    log_pages: LogPageRegistry,
    /// Robot timing model (pick/place/travel constants).
    timing: RobotTimingModel,
    /// Shared simulation clock (controls speed factor).
    clock: Arc<SimulationClock>,
    /// Serializes robot operations (one-at-a-time, like a real robot arm).
    robot_busy: Mutex<()>,
    /// WebSocket broadcast sender for notifying the frontend of state changes.
    ws_tx: Option<broadcast::Sender<()>>,
}

impl MediaChanger {
    pub fn new(
        model: &str,
        serial: &str,
        num_drives: u16,
        num_slots: u16,
        media_barcodes: &[String],
        drives: Vec<Arc<dyn MediaLoadNotify>>,
        timing: RobotTimingModel,
        clock: Arc<SimulationClock>,
        ws_tx: Option<broadcast::Sender<()>>,
    ) -> Self {
        let vendor = "QUANTUM ";
        let product = format!("{:<16}", model);
        let product = &product[..16];
        let revision = "0100";

        let mut inq = vec![0u8; 96];
        // Byte 0: Peripheral qualifier (0) | Device type (08h = Medium Changer)
        inq[0] = 0x08;
        // Byte 1: RMB=1 (removable media)
        inq[1] = 0x80;
        // Byte 2: Version (06h = SPC-4)
        inq[2] = 0x06;
        // Byte 3: Response data format (2) | HiSup=1 → 0x12
        inq[3] = 0x12;
        // Byte 4: Additional length (96 - 5 = 91)
        inq[4] = 91;
        // Byte 5: SCCS=0
        inq[5] = 0x00;
        // Byte 6: BarC=1 (barcode scanner installed), bit 3
        inq[6] = 0x08;
        // Byte 7: CmdQue=1
        inq[7] = 0x02;
        // Bytes 8-15: Vendor identification
        inq[8..16].copy_from_slice(vendor.as_bytes());
        // Bytes 16-31: Product identification
        inq[16..32].copy_from_slice(product.as_bytes());
        // Bytes 32-35: Product revision level
        inq[32..36].copy_from_slice(revision.as_bytes());
        // Version descriptors starting at byte 58
        // SAM-5 (0x00A0)
        inq[58] = 0x00;
        inq[59] = 0xA0;
        // SPC-4 (0x0460)
        inq[60] = 0x04;
        inq[61] = 0x60;
        // SMC-3 (0x0480)
        inq[62] = 0x04;
        inq[63] = 0x80;

        let changer_state = ChangerState::new(num_drives, num_slots, 1, media_barcodes);

        let mode_pages = mode_pages::default_registry(
            changer_state.start_picker,
            changer_state.num_pickers,
            changer_state.start_slot,
            changer_state.num_slots,
            changer_state.start_iee,
            changer_state.num_iee,
            changer_state.start_drive,
            changer_state.num_drives,
        );

        let log_pages = log_pages::default_registry();

        Self {
            inquiry_data: inq,
            serial: serial.to_string(),
            vendor: vendor.to_string(),
            product: product.to_string(),
            state: Mutex::new(changer_state),
            drives,
            mode_pages,
            log_pages,
            timing,
            clock,
            robot_busy: Mutex::new(()),
            ws_tx,
        }
    }

    /// Create a snapshot for API / dashboard consumption.
    pub fn snapshot(&self) -> ChangerSnapshot {
        let st = self.state.lock().unwrap();
        ChangerSnapshot::from_state(&st, &self.vendor, &self.product, &self.serial)
    }

    /// Send a WebSocket notification to the frontend.
    fn notify_ws(&self) {
        if let Some(tx) = &self.ws_tx {
            let _ = tx.send(());
        }
    }

    /// Convert a SCSI element address to a physical rail position.
    /// Drives, I/E ports, and slots are all on the same linear rail — the SCSI
    /// address namespaces (0x0001, 0x0010, 0x0100, 0x1000) don't reflect physical
    /// proximity.  We map them into a single linear space:
    ///   drives 0..N, then I/E, then slots.
    fn element_rail_position(&self, addr: u16) -> u16 {
        let st = self.state.lock().unwrap();
        if addr >= st.start_drive && addr < st.start_drive + st.num_drives {
            // Drive element → position = drive index
            addr - st.start_drive
        } else if addr >= st.start_iee && addr < st.start_iee + st.num_iee {
            // I/E element → position after drives
            st.num_drives + (addr - st.start_iee)
        } else if addr >= st.start_slot && addr < st.start_slot + st.num_slots {
            // Storage element → position after drives + I/E
            st.num_drives + st.num_iee + (addr - st.start_slot)
        } else {
            // MTE / unknown → position 0
            0
        }
    }

    /// MOVE MEDIUM with robot serialization and timing.
    fn timed_move_medium(&self, cdb: &[u8]) -> ScsiResult {
        // 1. Lock state, validate, extract params, release state (fast-fail on errors)
        let params = {
            let st = self.state.lock().unwrap();
            match commands::move_medium::validate_move_medium(cdb, &st) {
                Ok(params) => params,
                Err(result) => return result,
            }
        };

        // Convert SCSI element addresses to physical rail positions for distance calc
        let src_pos = self.element_rail_position(params.source);
        let dst_pos = self.element_rail_position(params.dest);
        let slot_distance = src_pos.abs_diff(dst_pos);

        // 2. Lock robot_busy (serialize with other robot operations)
        let _robot = self.robot_busy.lock().unwrap();

        // 3. Compute timing (wall-clock = real_time / speed_factor)
        let real_secs = self.timing.estimate_move_sec(
            slot_distance,
            params.source_is_drive,
            params.dest_is_drive,
        );
        let speed = self.clock.speed_factor();
        let wall_secs = if speed.is_infinite() || speed <= 0.0 {
            0.0
        } else {
            real_secs / speed
        };

        // 4. Set Moving state with timing info
        {
            let mut st = self.state.lock().unwrap();
            st.library_state = state::LibraryState::Moving {
                source: params.source,
                dest: params.dest,
            };
            st.robot_started_at_ms = Some(Utc::now().timestamp_millis());
            st.robot_estimated_secs = Some(wall_secs);
        }
        self.notify_ws();

        // 5. Sleep for the computed duration (sleep_sync applies speed factor internally)
        self.clock.sleep_sync(Duration::from_secs_f64(real_secs));

        // 6. Re-lock state, re-validate, execute element transfer, then release lock.
        //    Drive notifications are deferred to avoid holding the lock during slow I/O.
        let (result, notifications) = {
            let mut st = self.state.lock().unwrap();
            let (result, notifications) = commands::move_medium::execute_move_medium(
                params.source,
                params.dest,
                &mut st,
                &self.drives,
            );

            // 7. Reset to Ready and clear timing fields (still under lock)
            st.library_state = state::LibraryState::Ready;
            st.robot_started_at_ms = None;
            st.robot_estimated_secs = None;

            (result, notifications)
        }; // state lock released here

        // 8. Apply drive notifications outside the lock (media_loaded opens tape files = slow)
        commands::move_medium::DriveNotification::apply_all(notifications);

        result
    }

    /// INITIALIZE ELEMENT STATUS with robot serialization and timing.
    fn timed_initialize_element_status(&self) -> ScsiResult {
        let num_elements = {
            let st = self.state.lock().unwrap();
            st.elements.len() as u32
        };

        // Serialize with other robot operations
        let _robot = self.robot_busy.lock().unwrap();

        // Compute timing (wall-clock = real_time / speed_factor)
        let real_secs = self.timing.estimate_scan_sec(num_elements);
        let speed = self.clock.speed_factor();
        let wall_secs = if speed.is_infinite() || speed <= 0.0 {
            0.0
        } else {
            real_secs / speed
        };

        // Set Scanning state with timing info
        {
            let mut st = self.state.lock().unwrap();
            st.library_state = state::LibraryState::Scanning;
            st.robot_started_at_ms = Some(Utc::now().timestamp_millis());
            st.robot_estimated_secs = Some(wall_secs);
        }
        self.notify_ws();

        // Simulate inventory scan time (sleep_sync applies speed factor internally)
        self.clock.sleep_sync(Duration::from_secs_f64(real_secs));

        // Reset to Ready
        {
            let mut st = self.state.lock().unwrap();
            st.library_state = state::LibraryState::Ready;
            st.robot_started_at_ms = None;
            st.robot_estimated_secs = None;
        }

        sense::good()
    }
}

impl ScsiDevice for MediaChanger {
    fn execute_command(&self, cdb: &[u8], data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode = format!("{:02X}h", opcode), "SMC command");

        // Commands that don't need mutable state
        match opcode {
            INQUIRY => {
                return commands::inquiry::handle_inquiry(
                    cdb,
                    &self.inquiry_data,
                    &self.serial,
                    &self.vendor,
                    &self.product,
                );
            }
            REPORT_LUNS => {
                return commands::report_luns::handle_report_luns(cdb);
            }
            _ => {}
        }

        // MOVE MEDIUM: validate → serialize → sleep → execute
        if opcode == MOVE_MEDIUM {
            return self.timed_move_medium(cdb);
        }

        // INITIALIZE ELEMENT STATUS: serialize → sleep → return GOOD
        if opcode == INITIALIZE_ELEMENT_STATUS || opcode == INIT_ELEMENT_STATUS_WITH_RANGE {
            return self.timed_initialize_element_status();
        }

        let mut st = self.state.lock().unwrap();

        // Commands that need mutable state
        match opcode {
            TEST_UNIT_READY => commands::control::handle_test_unit_ready(&mut st),
            REQUEST_SENSE => commands::control::handle_request_sense(cdb, &mut st),
            PREVENT_ALLOW_MEDIUM_REMOVAL => {
                commands::control::handle_prevent_allow_medium_removal(cdb, &mut st)
            }
            MODE_SENSE_6 => commands::mode::handle_mode_sense_6(cdb, &self.mode_pages),
            MODE_SENSE_10 => commands::mode::handle_mode_sense_10(cdb, &self.mode_pages),
            MODE_SELECT_6 => commands::mode::handle_mode_select_6(cdb, data_out),
            MODE_SELECT_10 => commands::mode::handle_mode_select_10(cdb, data_out),
            LOG_SENSE => commands::log::handle_log_sense(cdb, &self.log_pages),
            READ_ELEMENT_STATUS => {
                commands::element_status::handle_read_element_status(cdb, &st)
            }
            _ => {
                trace!(
                    opcode = format!("{:02X}h", opcode),
                    "unsupported SMC command"
                );
                sense::SenseBuilder::invalid_opcode().to_check_condition()
            }
        }
    }
}
