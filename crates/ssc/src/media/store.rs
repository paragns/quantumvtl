//! Per-media redb persistence + append-only data file for tape record payloads.
//!
//! Each virtual tape cartridge is backed by two files:
//! - `{barcode}.redb` — metadata (record index, partition stats, MAM, media properties)
//! - `{barcode}.data` — append-only raw record data bytes

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use super::geometry::LtoGeneration;
use super::mam::MamAttributes;
use super::tape::{RecordDescriptor, TapeMedia, TapePartition};

// ── redb table definitions ──────────────────────────────────────────────────

/// Scalar media metadata: key = field name, value = bincode bytes.
const MEDIA_META: TableDefinition<&str, &[u8]> = TableDefinition::new("media_meta");

/// Record index: key = (partition_idx, record_num), value = bincode RecordDescriptor.
const RECORDS: TableDefinition<(u32, u64), &[u8]> = TableDefinition::new("records");

/// Per-partition statistics: key = partition_idx, value = bincode PartitionStats.
const PARTITION_META: TableDefinition<u32, &[u8]> = TableDefinition::new("partition_meta");

/// MAM attributes: stored as a single bincode blob keyed by "mam".
const MAM: TableDefinition<&str, &[u8]> = TableDefinition::new("mam");

// ── Serializable helper structs ─────────────────────────────────────────────

/// Lightweight media metadata (no partitions, no records, no MAM).
#[derive(Debug, Serialize, Deserialize)]
pub struct MediaMeta {
    pub barcode: String,
    pub generation: LtoGeneration,
    pub write_protected: bool,
    pub worm: bool,
    pub optimization_done: bool,
    pub compression_enabled: bool,
    pub total_loads: u32,
    pub meters_processed: f64,
    pub partition_count: u32,
}

/// Per-partition byte counters.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PartitionStats {
    pub bytes_written_native: u64,
    pub bytes_written_compressed: u64,
    pub bytes_read_native: u64,
    #[serde(default)]
    pub bytes_read_compressed: u64,
}

// ── TapeStore ───────────────────────────────────────────────────────────────

/// Manages per-media redb database + append-only data file.
pub struct TapeStore {
    db: Database,
    data_file: File,
    #[allow(dead_code)]
    data_path: PathBuf,
    /// Current end-of-file offset for appends.
    data_len: u64,
}

impl TapeStore {
    /// Open (or create) the store for a given barcode in `data_dir`.
    pub fn open(data_dir: &Path, barcode: &str) -> io::Result<Self> {
        fs::create_dir_all(data_dir)?;

        let db_path = data_dir.join(format!("{}.redb", barcode));
        let data_path = data_dir.join(format!("{}.data", barcode));

        let db = Database::create(&db_path).map_err(|e| {
            io::Error::other(format!("redb create failed: {}", e))
        })?;

        let mut data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&data_path)?;

        let data_len = data_file.seek(SeekFrom::End(0))?;

        Ok(Self {
            db,
            data_file,
            data_path,
            data_len,
        })
    }

    // ── Data file I/O ───────────────────────────────────────────────────

    /// Append raw data bytes to the .data file. Returns (offset, length).
    /// Note: caller is responsible for flushing (I/O thread batches flushes).
    pub fn append_data(&mut self, data: &[u8]) -> io::Result<(u64, u32)> {
        let offset = self.data_len;
        let length = data.len() as u32;
        self.data_file.seek(SeekFrom::End(0))?;
        self.data_file.write_all(data)?;
        self.data_len += length as u64;
        Ok((offset, length))
    }

    /// Read record data from the .data file at the given offset and length.
    pub fn read_data(&mut self, offset: u64, length: u32) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; length as usize];
        self.data_file.seek(SeekFrom::Start(offset))?;
        self.data_file.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Fsync the .data file to ensure all written data is durable on disk.
    pub fn sync_data(&self) -> io::Result<()> {
        self.data_file.sync_data()
    }

    /// Truncate the .data file to `new_len` bytes.
    pub fn truncate_data(&mut self, new_len: u64) -> io::Result<()> {
        self.data_file.set_len(new_len)?;
        self.data_len = new_len;
        Ok(())
    }

    // ── Record index ────────────────────────────────────────────────────

    /// Persist a single record descriptor to the redb index.
    pub fn save_record(
        &self,
        partition: u32,
        index: u64,
        desc: &RecordDescriptor,
    ) -> io::Result<()> {
        let encoded =
            bincode::serialize(desc).map_err(io::Error::other)?;
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            let mut table = txn
                .open_table(RECORDS)
                .map_err(io::Error::other)?;
            table
                .insert((partition, index), encoded.as_slice())
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Remove all record entries for `partition` with index >= `from_index`.
    pub fn remove_records_from(&self, partition: u32, from_index: u64) -> io::Result<()> {
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            let mut table = txn
                .open_table(RECORDS)
                .map_err(io::Error::other)?;
            // Collect keys to remove (can't mutate while iterating)
            let keys: Vec<(u32, u64)> = table
                .range((partition, from_index)..=(partition, u64::MAX))
                .map_err(io::Error::other)?
                .map(|entry| {
                    let (k, _) = entry.unwrap();
                    (k.value().0, k.value().1)
                })
                .collect();
            for key in keys {
                table
                    .remove(key)
                    .map_err(io::Error::other)?;
            }
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Load all record descriptors for a partition, ordered by index.
    pub fn load_partition_records(&self, partition: u32) -> io::Result<Vec<RecordDescriptor>> {
        let txn = self
            .db
            .begin_read()
            .map_err(io::Error::other)?;
        let table = match txn.open_table(RECORDS) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(io::Error::other(e)),
        };
        let mut records = Vec::new();
        let iter = table
            .range((partition, 0)..=(partition, u64::MAX))
            .map_err(io::Error::other)?;
        for entry in iter {
            let (_, v) = entry.map_err(io::Error::other)?;
            let desc: RecordDescriptor = bincode::deserialize(v.value())
                .map_err(io::Error::other)?;
            records.push(desc);
        }
        Ok(records)
    }

    // ── Media metadata ──────────────────────────────────────────────────

    /// Persist scalar media metadata to the redb store.
    pub fn save_media_meta(&self, media: &TapeMedia) -> io::Result<()> {
        let meta = MediaMeta {
            barcode: media.barcode.clone(),
            generation: media.generation,
            write_protected: media.write_protected,
            worm: media.worm,
            optimization_done: media.optimization_done,
            compression_enabled: media.compression_enabled,
            total_loads: media.total_loads,
            meters_processed: media.meters_processed,
            partition_count: media.partitions.len() as u32,
        };
        let encoded =
            bincode::serialize(&meta).map_err(io::Error::other)?;
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            let mut table = txn
                .open_table(MEDIA_META)
                .map_err(io::Error::other)?;
            table
                .insert("meta", encoded.as_slice())
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Load scalar media metadata from the redb store.
    pub fn load_media_meta(&self) -> io::Result<Option<MediaMeta>> {
        let txn = self
            .db
            .begin_read()
            .map_err(io::Error::other)?;
        let table = match txn.open_table(MEDIA_META) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(io::Error::other(e)),
        };
        match table.get("meta") {
            Ok(Some(v)) => {
                let meta: MediaMeta = bincode::deserialize(v.value())
                    .map_err(io::Error::other)?;
                Ok(Some(meta))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(io::Error::other(e)),
        }
    }

    // ── Partition stats ─────────────────────────────────────────────────

    /// Persist partition byte counters.
    pub fn save_partition_stats(&self, idx: u32, partition: &TapePartition) -> io::Result<()> {
        let stats = PartitionStats {
            bytes_written_native: partition.bytes_written_native,
            bytes_written_compressed: partition.bytes_written_compressed,
            bytes_read_native: partition.bytes_read_native,
            bytes_read_compressed: partition.bytes_read_compressed,
        };
        let encoded =
            bincode::serialize(&stats).map_err(io::Error::other)?;
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            let mut table = txn
                .open_table(PARTITION_META)
                .map_err(io::Error::other)?;
            table
                .insert(idx, encoded.as_slice())
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Load partition byte counters.
    pub fn load_partition_stats(&self, idx: u32) -> io::Result<PartitionStats> {
        let txn = self
            .db
            .begin_read()
            .map_err(io::Error::other)?;
        let table = match txn.open_table(PARTITION_META) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(PartitionStats::default()),
            Err(e) => return Err(io::Error::other(e)),
        };
        match table.get(idx) {
            Ok(Some(v)) => {
                let stats: PartitionStats = bincode::deserialize(v.value())
                    .map_err(io::Error::other)?;
                Ok(stats)
            }
            Ok(None) => Ok(PartitionStats::default()),
            Err(e) => Err(io::Error::other(e)),
        }
    }

    // ── MAM ─────────────────────────────────────────────────────────────

    /// Persist all MAM attributes to the redb store as a single blob.
    pub fn save_mam(&self, mam: &MamAttributes) -> io::Result<()> {
        let encoded =
            bincode::serialize(mam).map_err(io::Error::other)?;
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            let mut table = txn
                .open_table(MAM)
                .map_err(io::Error::other)?;
            table
                .insert("mam", encoded.as_slice())
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Load MAM attributes from the redb store.
    pub fn load_mam(&self) -> io::Result<MamAttributes> {
        let txn = self
            .db
            .begin_read()
            .map_err(io::Error::other)?;
        let table = match txn.open_table(MAM) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(MamAttributes::default()),
            Err(e) => return Err(io::Error::other(e)),
        };
        match table.get("mam") {
            Ok(Some(v)) => {
                let mam: MamAttributes = bincode::deserialize(v.value())
                    .map_err(io::Error::other)?;
                Ok(mam)
            }
            Ok(None) => Ok(MamAttributes::default()),
            Err(e) => Err(io::Error::other(e)),
        }
    }

    // ── Bulk clear ──────────────────────────────────────────────────────

    /// Clear all records for a given partition from the redb index.
    pub fn clear_partition_records(&self, partition: u32) -> io::Result<()> {
        self.remove_records_from(partition, 0)
    }

    /// Clear all records for ALL partitions from the redb index.
    pub fn clear_all_records(&self) -> io::Result<()> {
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            // Delete and recreate the table to clear it efficiently
            txn.delete_table(RECORDS)
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }

    /// Clear all partition stats.
    pub fn clear_all_partition_stats(&self) -> io::Result<()> {
        let txn = self
            .db
            .begin_write()
            .map_err(io::Error::other)?;
        {
            txn.delete_table(PARTITION_META)
                .map_err(io::Error::other)?;
        }
        txn.commit()
            .map_err(io::Error::other)?;
        Ok(())
    }
}
