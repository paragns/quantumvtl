//! SSC (SCSI Stream Commands) — IBM LTO tape drive emulation.
//!
//! This crate emulates an IBM Ultrium LTO tape drive. It implements the
//! ScsiDevice trait from the iscsi-target crate to handle SCSI CDB dispatch.

pub mod commands;
pub mod events;
pub mod log_pages;
pub mod media;
pub mod mode_pages;
pub mod sense;
pub mod snapshot;
pub mod timing;
pub mod vpd;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use iscsi_target::{MediaLoadNotify, ScsiDevice, ScsiResult};
use tracing::{trace, warn};

use commands::opcodes::*;
use log_pages::{DriveStats, LogPageRegistry, SharedDriveStats};
use media::geometry::LtoGeneration;
use media::position;
use media::store::TapeStore;
use media::tape::{DriveMediaState, RecordDescriptor, TapeMedia};
pub use media::tape::{read_media_detail, MediaDetail, PartitionDetail};
use mode_pages::ModePageRegistry;
use snapshot::{DriveActivity, DriveSnapshot};

/// Internal drive state protected by a mutex.
struct DriveState {
    /// Loaded media + position tracking. None if drive is empty.
    media_state: Option<DriveMediaState>,
    /// Current drive activity.
    activity: DriveActivity,
    /// Backhitch counter for this mount.
    backhitch_count: u32,
}

/// A SCSI Tape Drive device emulating an IBM Ultrium LTO drive.
pub struct TapeDrive {
    /// Standard INQUIRY response data (96 bytes).
    inquiry_data: Vec<u8>,
    /// Drive serial number.
    serial: String,
    /// LTO generation this drive emulates.
    generation: LtoGeneration,
    /// Directory for persisting tape data files.
    data_dir: PathBuf,
    /// Protected internal state.
    state: Mutex<DriveState>,
    /// Mode page registry.
    mode_pages: ModePageRegistry,
    /// Log page registry.
    log_pages: LogPageRegistry,
    /// Shared compression-enable flag (driven by Mode Page 0Fh).
    compression_enabled: Arc<AtomicBool>,
    /// Shared drive statistics (consumed by live log pages).
    drive_stats: SharedDriveStats,
}

impl TapeDrive {
    pub fn new(serial: &str, generation: LtoGeneration, data_dir: PathBuf) -> Self {
        let suffix = generation.product_suffix();
        let product = format!("ULT3580-{:<12}", suffix);
        let vendor = "IBM     ";
        let revision = "A1B0";

        let mut inq = vec![0u8; 96];
        // Byte 0: Peripheral qualifier (0) | Peripheral device type (0x01 = Sequential Access)
        inq[0] = 0x01;
        // Byte 1: RMB=1 (removable media)
        inq[1] = 0x80;
        // Byte 2: Version (0x06 = SPC-4)
        inq[2] = 0x06;
        // Byte 3: Response data format (2) | HiSup=1 → 0x12
        inq[3] = 0x12;
        // Byte 4: Additional length (96 - 5 = 91)
        inq[4] = 91;
        // Byte 7: CmdQue=1
        inq[7] = 0x02;
        // Bytes 8-15: Vendor identification
        inq[8..16].copy_from_slice(vendor.as_bytes());
        // Bytes 16-31: Product identification (16 bytes, space-padded)
        let product_bytes = product.as_bytes();
        let copy_len = product_bytes.len().min(16);
        inq[16..16 + copy_len].copy_from_slice(&product_bytes[..copy_len]);
        for i in 16 + copy_len..32 {
            inq[i] = 0x20; // space-pad
        }
        // Bytes 32-35: Product revision level
        inq[32..36].copy_from_slice(revision.as_bytes());
        // Version descriptors starting at byte 58
        // SAM-5 (0x00A0)
        inq[58] = 0x00;
        inq[59] = 0xA0;
        // SPC-4 (0x0460)
        inq[60] = 0x04;
        inq[61] = 0x60;
        // SSC-4 (0x0560)
        inq[62] = 0x05;
        inq[63] = 0x60;

        let dce = Arc::new(AtomicBool::new(true)); // default: compression enabled
        let drive_stats: SharedDriveStats = Arc::new(Mutex::new(DriveStats::default()));

        Self {
            inquiry_data: inq,
            serial: serial.to_string(),
            generation,
            data_dir,
            state: Mutex::new(DriveState {
                media_state: None,
                activity: DriveActivity::Empty,
                backhitch_count: 0,
            }),
            mode_pages: mode_pages::default_registry(dce.clone()),
            log_pages: log_pages::default_registry(drive_stats.clone()),
            compression_enabled: dce,
            drive_stats,
        }
    }

    /// Refresh shared DriveStats from current media state.
    /// Must be called with DriveState already locked.
    fn refresh_stats(drive_stats: &SharedDriveStats, media_state: Option<&DriveMediaState>, generation: LtoGeneration) {
        let mut s = drive_stats.lock().unwrap();
        match media_state {
            Some(ms) => {
                let geometry = generation.geometry();
                s.media_loaded = true;
                s.native_capacity_bytes = geometry.native_capacity_bytes;
                s.buffer_size_bytes = geometry.buffer_size_bytes;
                s.total_bytes_written_native = ms.media.total_native_bytes_written();
                s.total_bytes_written_compressed = ms.media.total_compressed_bytes_written();
                s.total_bytes_read_native = ms.media.total_native_bytes_read();
                s.total_bytes_read_compressed = ms.media.total_bytes_read_compressed();
                s.partition_count = ms.media.partitions.len() as u8;
                s.partition_native_written = ms.media.partitions.iter()
                    .map(|p| p.bytes_written_native).collect();
                s.partition_compressed_written = ms.media.partitions.iter()
                    .map(|p| p.bytes_written_compressed).collect();
                s.partition_remaining_bytes = ms.media.partitions.iter()
                    .map(|p| {
                        let per_partition_cap = geometry.native_capacity_bytes
                            / ms.media.partitions.len() as u64;
                        per_partition_cap.saturating_sub(p.bytes_written_native)
                    }).collect();
                s.total_loads = ms.media.total_loads;
                s.compression_enabled = ms.media.compression_enabled;
            }
            None => {
                *s = DriveStats::default();
            }
        }
    }

    /// Create a drive snapshot for API / dashboard consumption.
    pub fn snapshot(&self) -> DriveSnapshot {
        let st = self.state.lock().unwrap();
        match &st.media_state {
            Some(ms) => {
                let bytes_before = ms.current_partition().bytes_before_position(
                    ms.position.block_number,
                );
                let phys = position::logical_to_physical(
                    bytes_before,
                    ms.media.geometry.native_capacity_bytes,
                    ms.media.geometry,
                );
                DriveSnapshot {
                    serial: self.serial.clone(),
                    generation: self.generation,
                    loaded: true,
                    barcode: Some(ms.media.barcode.clone()),
                    write_protected: ms.media.write_protected,
                    worm: ms.media.worm,
                    partition: ms.position.partition,
                    block_number: ms.position.block_number,
                    file_number: ms.position.file_number,
                    at_bop: ms.at_bop(),
                    at_eod: ms.at_eod(),
                    current_wrap: Some(phys.wrap),
                    total_wraps: Some(ms.media.geometry.num_wraps),
                    position_in_wrap_pct: Some(phys.offset_in_wrap_pct),
                    write_buffer_pct: 0.0,
                    read_cache_pct: 0.0,
                    objects_in_buffer: 0,
                    buffer_state: "idle".into(),
                    drive_state: st.activity.clone(),
                    tape_speed: None,
                    operation_progress_pct: None,
                    instantaneous_rate_bytes_sec: None,
                    compression_ratio: if ms.media.compression_enabled {
                        Some(ms.media.compression_ratio())
                    } else {
                        None
                    },
                    backhitch_count_this_mount: st.backhitch_count,
                    capacity_used_pct: Some(ms.media.capacity_used_fraction() * 100.0),
                    native_bytes_written: ms.media.total_native_bytes_written(),
                    compressed_bytes_written: ms.media.total_compressed_bytes_written(),
                    approximate_remaining_mb: Some(ms.media.approximate_remaining_mb()),
                    total_loads: ms.media.total_loads,
                    motion_hours: 0.0,
                    // Legacy fields
                    position: ms.position.block_number as usize,
                    record_count: ms.current_partition().records.len(),
                }
            }
            None => DriveSnapshot::empty(&self.serial, self.generation),
        }
    }

    /// Return a live `MediaDetail` from the in-memory state if media is loaded.
    pub fn media_detail(&self) -> Option<MediaDetail> {
        let st = self.state.lock().unwrap();
        let ms = st.media_state.as_ref()?;
        let m = &ms.media;

        let partitions: Vec<PartitionDetail> = m
            .partitions
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let mut filemark_positions = Vec::new();
                let mut data_record_sizes = Vec::new();
                for (i, rec) in p.records.iter().enumerate() {
                    match rec {
                        RecordDescriptor::Filemark => filemark_positions.push(i as u64),
                        RecordDescriptor::Data { length, .. } => data_record_sizes.push(*length),
                        RecordDescriptor::CompressedData { compressed_length, .. } => data_record_sizes.push(*compressed_length),
                    }
                }
                PartitionDetail {
                    index: idx as u8,
                    record_count: p.records.len() as u64,
                    filemark_count: filemark_positions.len() as u64,
                    filemark_positions,
                    data_record_sizes,
                    bytes_written_native: p.bytes_written_native,
                    bytes_written_compressed: p.bytes_written_compressed,
                    bytes_read_native: p.bytes_read_native,
                }
            })
            .collect();

        let total_records = partitions.iter().map(|p| p.record_count).sum();
        let total_filemarks = partitions.iter().map(|p| p.filemark_count).sum();

        Some(MediaDetail {
            barcode: m.barcode.clone(),
            generation: m.generation,
            write_protected: m.write_protected,
            worm: m.worm,
            partition_count: partitions.len() as u8,
            total_records,
            total_filemarks,
            native_bytes_written: m.total_native_bytes_written(),
            compressed_bytes_written: m.total_compressed_bytes_written(),
            native_capacity_bytes: m.native_capacity_bytes(),
            capacity_used_pct: m.capacity_used_fraction() * 100.0,
            approximate_remaining_mb: m.approximate_remaining_mb(),
            compression_enabled: m.compression_enabled,
            compression_ratio: m.compression_ratio(),
            total_loads: m.total_loads,
            optimization_done: m.optimization_done,
            partitions,
        })
    }
}

impl MediaLoadNotify for TapeDrive {
    fn media_loaded(&self, barcode: &str) {
        let mut st = self.state.lock().unwrap();
        trace!(barcode, "tape media loaded into drive");

        // Open the per-media store (redb + data file)
        let store = match TapeStore::open(&self.data_dir, barcode) {
            Ok(s) => s,
            Err(e) => {
                warn!(barcode, error = %e, "failed to open tape store, drive stays empty");
                return;
            }
        };

        // Try to load existing media metadata from store
        let mut media = match store.load_media_meta() {
            Ok(Some(meta)) => {
                trace!(barcode, "restoring tape media from redb store");
                let generation = meta.generation;
                let geometry = generation.geometry();
                let mam = store.load_mam().unwrap_or_else(|e| {
                    warn!(barcode, error = %e, "failed to load MAM, using default");
                    media::mam::MamAttributes::new_for_cartridge(
                        barcode,
                        "IBM",
                        &format!("{:0>10}", barcode),
                    )
                });

                // Reconstruct partitions from stored records + stats
                let mut partitions = Vec::with_capacity(meta.partition_count as usize);
                for idx in 0..meta.partition_count {
                    let records = store.load_partition_records(idx).unwrap_or_else(|e| {
                        warn!(barcode, partition = idx, error = %e, "failed to load partition records");
                        Vec::new()
                    });
                    let stats = store.load_partition_stats(idx).unwrap_or_else(|e| {
                        warn!(barcode, partition = idx, error = %e, "failed to load partition stats");
                        media::store::PartitionStats::default()
                    });
                    let mut partition = media::tape::TapePartition {
                        records,
                        filemark_positions: Vec::new(),
                        bytes_written_native: stats.bytes_written_native,
                        bytes_written_compressed: stats.bytes_written_compressed,
                        bytes_read_native: stats.bytes_read_native,
                        bytes_read_compressed: stats.bytes_read_compressed,
                    };
                    partition.rebuild_filemark_index();
                    partitions.push(partition);
                }
                if partitions.is_empty() {
                    partitions.push(media::tape::TapePartition::new());
                }

                TapeMedia {
                    barcode: meta.barcode,
                    generation,
                    geometry,
                    partitions,
                    write_protected: meta.write_protected,
                    worm: meta.worm,
                    mam,
                    optimization_done: meta.optimization_done,
                    compression_enabled: meta.compression_enabled,
                    total_loads: meta.total_loads,
                    meters_processed: meta.meters_processed,
                }
            }
            Ok(None) => {
                // New tape — no existing metadata
                trace!(barcode, "creating blank tape media");
                TapeMedia::new(barcode, self.generation)
            }
            Err(e) => {
                warn!(barcode, error = %e, "failed to load media meta, creating blank");
                TapeMedia::new(barcode, self.generation)
            }
        };

        media.record_load();

        // Sync compression_enabled AtomicBool from media state
        self.compression_enabled
            .store(media.compression_enabled, Ordering::Relaxed);

        // Set MAM capacity attributes (8-byte big-endian MB values)
        let cap_mb = media.native_capacity_bytes() / 1_000_000;
        let remaining_mb = media.approximate_remaining_mb();
        media.mam.set_device_managed(0x0000, remaining_mb.to_be_bytes().to_vec());
        media.mam.set_device_managed(0x0001, cap_mb.to_be_bytes().to_vec());

        // Persist the initial metadata (including updated load count)
        if let Err(e) = store.save_media_meta(&media) {
            warn!(barcode, error = %e, "failed to persist initial media metadata");
        }
        if let Err(e) = store.save_mam(&media.mam) {
            warn!(barcode, error = %e, "failed to persist MAM on load");
        }

        st.media_state = Some(DriveMediaState::new(media, store));
        st.activity = DriveActivity::Idle;
        st.backhitch_count = 0;

        // Refresh shared stats for log pages
        Self::refresh_stats(&self.drive_stats, st.media_state.as_ref(), self.generation);
    }

    fn media_unloaded(&self) {
        let mut st = self.state.lock().unwrap();
        trace!("tape media unloaded from drive");

        // Flush final metadata to store before dropping
        if let Some(ref mut ms) = st.media_state {
            let barcode = ms.media.barcode.clone();

            // Sync compression flag from AtomicBool back to media
            ms.media.compression_enabled = self.compression_enabled.load(Ordering::Relaxed);

            // Update MAM capacity attributes (8-byte big-endian MB values)
            let remaining_mb = ms.media.approximate_remaining_mb();
            ms.media.mam.set_device_managed(0x0000, remaining_mb.to_be_bytes().to_vec());
            let written_mb = ms.media.total_native_bytes_written() / 1_000_000;
            ms.media.mam.set_device_managed(0x0220, written_mb.to_be_bytes().to_vec());
            let read_mb = ms.media.total_native_bytes_read() / 1_000_000;
            ms.media.mam.set_device_managed(0x0222, read_mb.to_be_bytes().to_vec());

            if let Err(e) = ms.store.save_media_meta(&ms.media) {
                warn!(barcode, error = %e, "failed to persist media metadata on unload");
            }
            for (idx, partition) in ms.media.partitions.iter().enumerate() {
                if let Err(e) = ms.store.save_partition_stats(idx as u32, partition) {
                    warn!(barcode, partition = idx, error = %e, "failed to persist partition stats on unload");
                }
            }
            if let Err(e) = ms.store.save_mam(&ms.media.mam) {
                warn!(barcode, error = %e, "failed to persist MAM on unload");
            }
            trace!(barcode, "flushed tape metadata to redb store");
        }

        // Update shared stats (media unloaded)
        Self::refresh_stats(&self.drive_stats, None, self.generation);

        // Drop the DriveMediaState — redb + file handles close automatically
        st.media_state = None;
        st.activity = DriveActivity::Empty;
    }
}

impl ScsiDevice for TapeDrive {
    fn execute_command(&self, cdb: &[u8], data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode = format!("{:02X}h", opcode), "SSC command");

        // Commands that don't require media
        match opcode {
            INQUIRY => {
                return commands::inquiry::handle_inquiry(cdb, &self.inquiry_data, &self.serial);
            }
            REQUEST_SENSE => {
                return commands::control::handle_request_sense(cdb);
            }
            REPORT_DENSITY_SUPPORT => {
                return commands::density::handle_report_density_support(cdb);
            }
            MAINTENANCE_IN => {
                return commands::maintenance::handle_maintenance_in(cdb);
            }
            MAINTENANCE_OUT => {
                return commands::maintenance::handle_maintenance_out(cdb, data_out);
            }
            _ => {}
        }

        let mut st = self.state.lock().unwrap();

        // TEST UNIT READY — special: check media but no media_state borrow
        if opcode == TEST_UNIT_READY {
            return commands::control::handle_test_unit_ready(st.media_state.is_some());
        }

        // Commands that require media (LOG/MODE commands can work without but we simplify)
        match opcode {
            // Mode page commands — work even without media
            MODE_SENSE_6 => {
                let wp = st.media_state.as_ref().map_or(false, |ms| ms.media.write_protected);
                return commands::mode::handle_mode_sense_6(cdb, &self.mode_pages, wp);
            }
            MODE_SENSE_10 => {
                let wp = st.media_state.as_ref().map_or(false, |ms| ms.media.write_protected);
                return commands::mode::handle_mode_sense_10(cdb, &self.mode_pages, wp);
            }
            MODE_SELECT_6 => {
                return commands::mode::handle_mode_select_6(cdb, data_out, &self.mode_pages);
            }
            MODE_SELECT_10 => {
                return commands::mode::handle_mode_select_10(cdb, data_out, &self.mode_pages);
            }
            LOG_SENSE => {
                return commands::log::handle_log_sense(cdb, &self.log_pages);
            }
            LOG_SELECT => {
                return commands::log::handle_log_select(cdb, &self.log_pages);
            }
            LOAD_UNLOAD => {
                return commands::control::handle_load_unload(cdb);
            }
            PREVENT_ALLOW_MEDIUM_REMOVAL => {
                return commands::control::handle_prevent_allow_medium_removal(cdb);
            }
            _ => {}
        }

        // All remaining commands require media to be loaded
        let media_state = match st.media_state.as_mut() {
            Some(ms) => ms,
            None => return sense::SenseBuilder::no_media().to_check_condition(),
        };

        let result = match opcode {
            REWIND => commands::position::handle_rewind(media_state),
            READ_6 => commands::read::handle_read_6(cdb, media_state),
            WRITE_6 => {
                // Sync compression flag from AtomicBool before writing
                media_state.media.compression_enabled =
                    self.compression_enabled.load(Ordering::Relaxed);
                commands::write::handle_write_6(cdb, data_out, media_state)
            }
            WRITE_FILEMARKS_6 => commands::filemarks::handle_write_filemarks(cdb, media_state),
            SPACE_6 => commands::position::handle_space_6(cdb, media_state),
            SPACE_16 => commands::position::handle_space_16(cdb, media_state),
            READ_POSITION => commands::position::handle_read_position(cdb, media_state),
            READ_BLOCK_LIMITS => commands::position::handle_read_block_limits(media_state),
            LOCATE_10 => commands::position::handle_locate_10(cdb, media_state),
            LOCATE_16 => commands::position::handle_locate_16(cdb, media_state),
            ERASE_6 => commands::erase::handle_erase(cdb, media_state),
            FORMAT_MEDIUM => commands::erase::handle_format_medium(cdb, media_state),
            ALLOW_OVERWRITE => commands::control::handle_allow_overwrite(cdb),
            READ_ATTRIBUTE => commands::attribute::handle_read_attribute(cdb, &media_state.media.mam),
            WRITE_ATTRIBUTE => {
                commands::attribute::handle_write_attribute(cdb, data_out, &mut media_state.media.mam)
            }
            _ => {
                trace!(opcode = format!("{:02X}h", opcode), "unsupported SSC command");
                sense::SenseBuilder::invalid_opcode().to_check_condition()
            }
        };

        // Refresh shared stats after data-path commands
        match opcode {
            WRITE_6 | READ_6 | WRITE_FILEMARKS_6 | SPACE_6 | SPACE_16 | ERASE_6
            | FORMAT_MEDIUM => {
                Self::refresh_stats(
                    &self.drive_stats,
                    st.media_state.as_ref(),
                    self.generation,
                );
            }
            _ => {}
        }

        result
    }
}
