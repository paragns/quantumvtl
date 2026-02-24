//! Tape media model — cartridge content, partitions, and records.

use super::geometry::{LtoGeneration, TapeGeometry};
use super::mam::MamAttributes;
use super::position::LogicalPosition;
use serde::{Deserialize, Serialize};

/// Default geometry placeholder for serde deserialization (overwritten by `fix_geometry`).
fn default_geometry() -> &'static TapeGeometry {
    LtoGeneration::Lto9.geometry()
}

/// Lightweight record metadata — no data payload.
/// Data blocks store their offset and length in the .data file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordDescriptor {
    /// Data block: offset and length in the .data file.
    Data { offset: u64, length: u32 },
    /// Filemark (no data).
    Filemark,
}

impl RecordDescriptor {
    /// Size of this record in bytes (0 for filemarks).
    pub fn byte_len(&self) -> u32 {
        match self {
            RecordDescriptor::Data { length, .. } => *length,
            RecordDescriptor::Filemark => 0,
        }
    }

    pub fn is_filemark(&self) -> bool {
        matches!(self, RecordDescriptor::Filemark)
    }
}

/// A single partition on tape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapePartition {
    /// Sequential records in this partition.
    pub records: Vec<RecordDescriptor>,
    /// Fast lookup of filemark positions (indices into records vec).
    pub filemark_positions: Vec<u64>,
    /// Total uncompressed bytes written to this partition.
    pub bytes_written_native: u64,
    /// Total "compressed" bytes (simulated) written to this partition.
    pub bytes_written_compressed: u64,
    /// Total uncompressed bytes read from this partition.
    pub bytes_read_native: u64,
}

impl TapePartition {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            filemark_positions: Vec::new(),
            bytes_written_native: 0,
            bytes_written_compressed: 0,
            bytes_read_native: 0,
        }
    }

    /// Logical object count (records + filemarks) in this partition.
    pub fn object_count(&self) -> u64 {
        self.records.len() as u64
    }

    /// Count of filemarks in this partition.
    pub fn filemark_count(&self) -> u64 {
        self.filemark_positions.len() as u64
    }

    /// Native bytes written before the given block position.
    pub fn bytes_before_position(&self, block_number: u64) -> u64 {
        let n = (block_number as usize).min(self.records.len());
        self.records[..n].iter().map(|r| r.byte_len() as u64).sum()
    }

    /// Rebuild the filemark position index.
    pub fn rebuild_filemark_index(&mut self) {
        self.filemark_positions = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_filemark())
            .map(|(i, _)| i as u64)
            .collect();
    }
}

impl Default for TapePartition {
    fn default() -> Self {
        Self::new()
    }
}

/// A virtual tape cartridge with its content and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeMedia {
    /// Barcode label (e.g., "ABC001L9").
    pub barcode: String,
    /// LTO generation of this cartridge.
    pub generation: LtoGeneration,
    /// Reference to geometry constants (reconstructed from `generation` on deserialize).
    #[serde(skip, default = "default_geometry")]
    pub geometry: &'static TapeGeometry,
    /// Partitions (at least one; up to max_partitions).
    pub partitions: Vec<TapePartition>,
    /// Whether the cartridge is write-protected.
    pub write_protected: bool,
    /// Whether this is a WORM cartridge.
    pub worm: bool,
    /// Medium Auxiliary Memory.
    pub mam: MamAttributes,
    /// Whether LTO-9 media optimization has been completed.
    pub optimization_done: bool,
    /// Compression enabled flag (driven by mode page).
    pub compression_enabled: bool,
    /// Simulated compression ratio (e.g., 2.5 for 2.5:1).
    pub compression_ratio: f64,
    /// Total lifetime load count.
    pub total_loads: u32,
    /// Total meters of tape processed (simulated).
    pub meters_processed: f64,
}

/// Snapshot for serialization to API / dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct TapeMediaSnapshot {
    pub barcode: String,
    pub generation: LtoGeneration,
    pub write_protected: bool,
    pub worm: bool,
    pub partition_count: u8,
    pub total_records: u64,
    pub native_bytes_written: u64,
    pub compressed_bytes_written: u64,
    pub native_capacity_bytes: u64,
    pub compression_enabled: bool,
    pub compression_ratio: f64,
    pub total_loads: u32,
    pub optimization_done: bool,
}

/// Detailed per-partition information for the media detail API.
#[derive(Debug, Clone, Serialize)]
pub struct PartitionDetail {
    pub index: u8,
    pub record_count: u64,
    pub filemark_count: u64,
    pub filemark_positions: Vec<u64>,
    pub data_record_sizes: Vec<u32>,
    pub bytes_written_native: u64,
    pub bytes_written_compressed: u64,
    pub bytes_read_native: u64,
}

/// Full media detail — everything known about a cartridge from its .redb store.
#[derive(Debug, Clone, Serialize)]
pub struct MediaDetail {
    pub barcode: String,
    pub generation: LtoGeneration,
    pub write_protected: bool,
    pub worm: bool,
    pub partition_count: u8,
    pub total_records: u64,
    pub total_filemarks: u64,
    pub native_bytes_written: u64,
    pub compressed_bytes_written: u64,
    pub native_capacity_bytes: u64,
    pub capacity_used_pct: f64,
    pub approximate_remaining_mb: u64,
    pub compression_enabled: bool,
    pub compression_ratio: f64,
    pub total_loads: u32,
    pub optimization_done: bool,
    pub partitions: Vec<PartitionDetail>,
}

/// Read detailed media information directly from the .redb store file.
/// Does not require the media to be loaded in a drive.
pub fn read_media_detail(data_dir: &std::path::Path, barcode: &str) -> Option<MediaDetail> {
    let store = super::store::TapeStore::open(data_dir, barcode).ok()?;
    let meta = store.load_media_meta().ok()??;

    let generation = meta.generation;
    let geometry = generation.geometry();

    let mut partitions = Vec::with_capacity(meta.partition_count as usize);
    let mut total_records: u64 = 0;
    let mut total_filemarks: u64 = 0;
    let mut native_bytes_written: u64 = 0;
    let mut compressed_bytes_written: u64 = 0;

    for idx in 0..meta.partition_count {
        let records = store.load_partition_records(idx).unwrap_or_default();
        let stats = store
            .load_partition_stats(idx)
            .unwrap_or_default();

        let mut filemark_positions = Vec::new();
        let mut data_record_sizes = Vec::new();
        for (i, rec) in records.iter().enumerate() {
            match rec {
                RecordDescriptor::Filemark => {
                    filemark_positions.push(i as u64);
                }
                RecordDescriptor::Data { length, .. } => {
                    data_record_sizes.push(*length);
                }
            }
        }

        let record_count = records.len() as u64;
        let filemark_count = filemark_positions.len() as u64;
        total_records += record_count;
        total_filemarks += filemark_count;
        native_bytes_written += stats.bytes_written_native;
        compressed_bytes_written += stats.bytes_written_compressed;

        partitions.push(PartitionDetail {
            index: idx as u8,
            record_count,
            filemark_count,
            filemark_positions,
            data_record_sizes,
            bytes_written_native: stats.bytes_written_native,
            bytes_written_compressed: stats.bytes_written_compressed,
            bytes_read_native: stats.bytes_read_native,
        });
    }

    // If no partitions were stored, there's at least one empty partition
    if partitions.is_empty() {
        partitions.push(PartitionDetail {
            index: 0,
            record_count: 0,
            filemark_count: 0,
            filemark_positions: Vec::new(),
            data_record_sizes: Vec::new(),
            bytes_written_native: 0,
            bytes_written_compressed: 0,
            bytes_read_native: 0,
        });
    }

    let native_capacity = geometry.native_capacity_bytes;
    let used_pct = if native_capacity > 0 {
        (native_bytes_written as f64 / native_capacity as f64 * 100.0).min(100.0)
    } else {
        0.0
    };
    let remaining_mb = native_capacity.saturating_sub(native_bytes_written) / 1_000_000;

    Some(MediaDetail {
        barcode: meta.barcode,
        generation,
        write_protected: meta.write_protected,
        worm: meta.worm,
        partition_count: partitions.len() as u8,
        total_records,
        total_filemarks,
        native_bytes_written,
        compressed_bytes_written,
        native_capacity_bytes: native_capacity,
        capacity_used_pct: used_pct,
        approximate_remaining_mb: remaining_mb,
        compression_enabled: meta.compression_enabled,
        compression_ratio: meta.compression_ratio,
        total_loads: meta.total_loads,
        optimization_done: meta.optimization_done,
        partitions,
    })
}

impl TapeMedia {
    /// Create a new blank tape for the given generation.
    pub fn new(barcode: &str, generation: LtoGeneration) -> Self {
        let geometry = generation.geometry();
        let mam = MamAttributes::new_for_cartridge(barcode, "IBM", &format!("{:0>10}", barcode));

        Self {
            barcode: barcode.to_string(),
            generation,
            geometry,
            partitions: vec![TapePartition::new()],
            write_protected: false,
            worm: false,
            mam,
            optimization_done: !geometry.requires_media_optimization,
            compression_enabled: true,
            compression_ratio: 2.5,
            total_loads: 0,
            meters_processed: 0.0,
        }
    }

    /// Reconstruct the geometry reference from the generation field after deserialization.
    pub fn fix_geometry(&mut self) {
        self.geometry = self.generation.geometry();
    }

    /// The currently active partition.
    pub fn partition(&self, index: u8) -> Option<&TapePartition> {
        self.partitions.get(index as usize)
    }

    /// The currently active partition (mutable).
    pub fn partition_mut(&mut self, index: u8) -> Option<&mut TapePartition> {
        self.partitions.get_mut(index as usize)
    }

    /// Native capacity of this cartridge in bytes.
    pub fn native_capacity_bytes(&self) -> u64 {
        self.geometry.native_capacity_bytes
    }

    /// Approximate native bytes written across all partitions.
    pub fn total_native_bytes_written(&self) -> u64 {
        self.partitions.iter().map(|p| p.bytes_written_native).sum()
    }

    /// Approximate compressed bytes written across all partitions.
    pub fn total_compressed_bytes_written(&self) -> u64 {
        self.partitions
            .iter()
            .map(|p| p.bytes_written_compressed)
            .sum()
    }

    /// Approximate remaining native capacity in bytes.
    pub fn approximate_remaining_bytes(&self) -> u64 {
        let used = self.total_native_bytes_written();
        self.native_capacity_bytes().saturating_sub(used)
    }

    /// Approximate remaining capacity in MB (10^6).
    pub fn approximate_remaining_mb(&self) -> u64 {
        self.approximate_remaining_bytes() / 1_000_000
    }

    /// Capacity used as a fraction (0.0 to 1.0).
    pub fn capacity_used_fraction(&self) -> f64 {
        let cap = self.native_capacity_bytes() as f64;
        if cap == 0.0 {
            return 0.0;
        }
        (self.total_native_bytes_written() as f64 / cap).min(1.0)
    }

    /// Record a load event.
    pub fn record_load(&mut self) {
        self.total_loads += 1;
        self.mam.increment_load_count();
    }

    /// Create a snapshot for API/dashboard use.
    pub fn snapshot(&self) -> TapeMediaSnapshot {
        TapeMediaSnapshot {
            barcode: self.barcode.clone(),
            generation: self.generation,
            write_protected: self.write_protected,
            worm: self.worm,
            partition_count: self.partitions.len() as u8,
            total_records: self.partitions.iter().map(|p| p.object_count()).sum(),
            native_bytes_written: self.total_native_bytes_written(),
            compressed_bytes_written: self.total_compressed_bytes_written(),
            native_capacity_bytes: self.native_capacity_bytes(),
            compression_enabled: self.compression_enabled,
            compression_ratio: self.compression_ratio,
            total_loads: self.total_loads,
            optimization_done: self.optimization_done,
        }
    }
}

/// Drive-internal state: media + position tracking + persistence.
pub struct DriveMediaState {
    pub media: TapeMedia,
    pub position: LogicalPosition,
    pub store: super::store::TapeStore,
}

impl DriveMediaState {
    pub fn new(media: TapeMedia, store: super::store::TapeStore) -> Self {
        Self {
            media,
            position: LogicalPosition::default(),
            store,
        }
    }

    /// Current partition reference.
    pub fn current_partition(&self) -> &TapePartition {
        self.media
            .partition(self.position.partition)
            .expect("active partition must exist")
    }

    /// Current partition mutable reference.
    pub fn current_partition_mut(&mut self) -> &mut TapePartition {
        self.media
            .partition_mut(self.position.partition)
            .expect("active partition must exist")
    }

    /// Whether we're at end-of-data in the current partition.
    pub fn at_eod(&self) -> bool {
        self.position.block_number >= self.current_partition().object_count()
    }

    /// Whether we're at beginning-of-partition.
    pub fn at_bop(&self) -> bool {
        self.position.at_bop()
    }

    /// Total object count in current partition.
    pub fn current_partition_objects(&self) -> u64 {
        self.current_partition().object_count()
    }
}
