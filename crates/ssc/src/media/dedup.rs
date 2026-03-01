//! Sharded deduplication pool with write-back cache.
//!
//! Unique 4KB blocks are stored in N append-only shard files
//! (`dedup_000.data` .. `dedup_NNN.data`). Shards are selected
//! pseudo-randomly via `fastrand`. Writes are ACK'd immediately after
//! inserting into an in-memory write-back cache; background flush
//! workers batch-write dirty blocks to shard files and unpin them.
//!
//! An in-memory `DashMap` provides duplicate detection on writes.
//! The write-back cache serves both dirty (pinned) and clean reads.

use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

use dashmap::DashMap;
use tracing::{info, warn};

use super::writeback_cache::WriteBackCache;

/// Block size for dedup (4KB).
pub const DEDUP_BLOCK_SIZE: usize = 4096;

/// 128-bit hash of a dedup block (stored as byte array for DashMap key).
pub type BlockHash = u128;

/// Pack a shard ID (0–255) and a byte offset within that shard into a single `u64`.
///
/// Layout: `[shard_id: 8 bits][offset: 56 bits]`
fn pack_ref(shard: u8, offset: u64) -> u64 {
    debug_assert!(offset < (1u64 << 56), "offset exceeds 56-bit range");
    ((shard as u64) << 56) | (offset & 0x00FF_FFFF_FFFF_FFFF)
}

/// Unpack a dedup reference into `(shard_id, offset)`.
fn unpack_ref(r: u64) -> (u8, u64) {
    let shard = (r >> 56) as u8;
    let offset = r & 0x00FF_FFFF_FFFF_FFFF;
    (shard, offset)
}

/// Statistics for the dedup pool.
#[derive(Debug, Default)]
pub struct DedupStats {
    /// Number of unique blocks stored on disk.
    pub blocks_stored: AtomicU64,
    /// Number of duplicate blocks detected (writes avoided).
    pub blocks_deduped: AtomicU64,
    /// Number of cache hits on read path.
    pub cache_hits: AtomicU64,
    /// Number of cache misses on read path.
    pub cache_misses: AtomicU64,
    /// Number of flush ops (large writes to shard files).
    pub flush_ops: AtomicU64,
    /// Total bytes flushed to disk.
    pub flush_bytes: AtomicU64,
}

/// An entry queued for a flush worker to write to a shard file.
struct WriteEntry {
    offset: u64,
    data: Box<[u8; DEDUP_BLOCK_SIZE]>,
    dedup_ref: u64,
}

/// A single shard: one append-only data file with separate read/write handles.
struct Shard {
    writer: Mutex<Option<File>>,
    reader: File,
    len: AtomicU64,
    flushed_len: AtomicU64,
    write_list: Mutex<Vec<WriteEntry>>,
    wake: Condvar,
}

/// Sharded dedup pool shared across all tapes.
///
/// This replaces the old single-file `DedupStore` with a sharded, write-back
/// cached design identical to the one in quantumvdisk.
pub struct DedupPool {
    shards: Arc<Vec<Shard>>,
    num_shards: usize,
    index: DashMap<BlockHash, u64>,
    cache: Arc<WriteBackCache>,
    path: PathBuf,
    pub stats: Arc<DedupStats>,
    shutdown: Arc<AtomicBool>,
    flush_workers: Mutex<Vec<JoinHandle<()>>>,
}

impl DedupPool {
    /// Open or create a dedup pool. Rebuilds the in-memory index by scanning
    /// existing shard data.
    pub fn open(
        data_dir: &Path,
        num_shards: usize,
        cache_bytes: usize,
        num_flush_workers: usize,
    ) -> io::Result<Self> {
        assert!(num_shards > 0 && num_shards <= 256, "num_shards must be 1..=256");
        assert!(num_flush_workers > 0, "need at least 1 flush worker");
        std::fs::create_dir_all(data_dir)?;

        let cache_blocks = (cache_bytes / DEDUP_BLOCK_SIZE).max(1);

        let mut shards = Vec::with_capacity(num_shards);
        let index: DashMap<BlockHash, u64> = DashMap::new();
        let mut total_blocks: u64 = 0;

        for shard_id in 0..num_shards {
            let path = shard_path(data_dir, shard_id);

            let writer = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&path)?;

            let reader = File::open(&path)?;

            let file_len = writer.metadata()?.len();

            if file_len % DEDUP_BLOCK_SIZE as u64 != 0 {
                warn!(
                    shard_id,
                    file_len,
                    "shard file has non-aligned length, truncating to block boundary"
                );
                let aligned = file_len / DEDUP_BLOCK_SIZE as u64 * DEDUP_BLOCK_SIZE as u64;
                writer.set_len(aligned)?;
            }

            let aligned_len = file_len / DEDUP_BLOCK_SIZE as u64 * DEDUP_BLOCK_SIZE as u64;
            let num_blocks = aligned_len / DEDUP_BLOCK_SIZE as u64;

            if num_blocks > 0 {
                info!(
                    shard_id,
                    num_blocks,
                    file_bytes = aligned_len,
                    "rebuilding dedup index for shard"
                );

                let mut scan_file = File::open(&path)?;
                let mut buf = [0u8; DEDUP_BLOCK_SIZE];

                for block_num in 0..num_blocks {
                    let offset = block_num * DEDUP_BLOCK_SIZE as u64;
                    scan_file.read_exact(&mut buf).map_err(|e| {
                        io::Error::new(
                            e.kind(),
                            format!(
                                "failed to read shard {} block {} at offset {}: {}",
                                shard_id, block_num, offset, e
                            ),
                        )
                    })?;
                    let hash = hash_block(&buf);
                    let packed = pack_ref(shard_id as u8, offset);
                    index.insert(hash, packed);
                }

                total_blocks += num_blocks;
            }

            shards.push(Shard {
                writer: Mutex::new(Some(writer)),
                reader,
                len: AtomicU64::new(aligned_len),
                flushed_len: AtomicU64::new(aligned_len),
                write_list: Mutex::new(Vec::new()),
                wake: Condvar::new(),
            });
        }

        if total_blocks > 0 {
            info!(
                unique_blocks = index.len(),
                total_blocks,
                num_shards,
                "dedup pool index rebuild complete"
            );
        }

        let cache = Arc::new(WriteBackCache::new(cache_blocks));
        let stats = Arc::new(DedupStats::default());
        let shutdown = Arc::new(AtomicBool::new(false));
        let shards = Arc::new(shards);

        // Spawn flush workers.
        let mut flush_handles = Vec::with_capacity(num_flush_workers);
        for worker_id in 0..num_flush_workers {
            let mut writers: Vec<(usize, File)> = Vec::new();
            for shard_id in 0..num_shards {
                if shard_id % num_flush_workers == worker_id {
                    let mut guard = shards[shard_id].writer.lock().unwrap();
                    let file = guard.take().expect("writer already taken");
                    writers.push((shard_id, file));
                }
            }

            let shards_clone = shards.clone();
            let cache_clone = cache.clone();
            let stats_clone = stats.clone();
            let shutdown_clone = shutdown.clone();

            let handle = thread::Builder::new()
                .name(format!("dedup-flush-{}", worker_id))
                .spawn(move || {
                    flush_worker(
                        &shards_clone,
                        writers,
                        &cache_clone,
                        &stats_clone,
                        &shutdown_clone,
                    );
                })
                .expect("failed to spawn flush worker");

            flush_handles.push(handle);
        }

        Ok(Self {
            shards,
            num_shards,
            index,
            cache,
            path: data_dir.to_path_buf(),
            stats,
            shutdown,
            flush_workers: Mutex::new(flush_handles),
        })
    }

    /// Store a block (≤ 4KB, zero-padded to 4KB). Returns a packed dedup reference.
    /// If the block already exists, returns the existing ref (no write).
    pub fn store_block(&self, data: &[u8]) -> io::Result<u64> {
        assert!(
            data.len() <= DEDUP_BLOCK_SIZE,
            "block exceeds DEDUP_BLOCK_SIZE"
        );

        // Zero-pad to 4KB
        let mut padded = [0u8; DEDUP_BLOCK_SIZE];
        padded[..data.len()].copy_from_slice(data);

        let hash = hash_block(&padded);

        // Check if block already exists (lock-free DashMap read)
        if let Some(existing) = self.index.get(&hash) {
            self.stats.blocks_deduped.fetch_add(1, Ordering::Relaxed);
            return Ok(*existing);
        }

        // Pick shard pseudo-randomly
        let shard_id = fastrand::usize(..self.num_shards);
        let shard = &self.shards[shard_id];

        // Pre-allocate offset atomically
        let offset = shard.len.fetch_add(DEDUP_BLOCK_SIZE as u64, Ordering::Relaxed);
        let packed = pack_ref(shard_id as u8, offset);

        // Double-check for races
        if let Some(existing) = self.index.get(&hash) {
            self.stats.blocks_deduped.fetch_add(1, Ordering::Relaxed);
            return Ok(*existing);
        }

        self.index.insert(hash, packed);

        // Insert into cache as PINNED
        self.cache.insert_pinned(packed, Box::new(padded));

        // Enqueue for flush worker
        {
            let mut wl = shard.write_list.lock().unwrap();
            wl.push(WriteEntry {
                offset,
                data: Box::new(padded),
                dedup_ref: packed,
            });
        }
        shard.wake.notify_one();

        self.stats.blocks_stored.fetch_add(1, Ordering::Relaxed);
        Ok(packed)
    }

    /// Read a block at the given dedup reference. Checks cache first.
    pub fn read_block(&self, dedup_ref: u64) -> io::Result<[u8; DEDUP_BLOCK_SIZE]> {
        if let Some(block) = self.cache.get(dedup_ref) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(block);
        }

        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);

        let (shard_id, offset) = unpack_ref(dedup_ref);
        let shard = &self.shards[shard_id as usize];

        let flushed = shard.flushed_len.load(Ordering::Relaxed);
        if offset + DEDUP_BLOCK_SIZE as u64 > flushed {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "block not yet flushed and not in cache (should be pinned)",
            ));
        }

        let mut buf = [0u8; DEDUP_BLOCK_SIZE];
        #[cfg(unix)]
        {
            shard.reader.read_at(&mut buf, offset)?;
        }
        #[cfg(not(unix))]
        {
            let _ = offset;
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "non-unix read not supported",
            ));
        }

        self.cache.insert_clean(dedup_ref, Box::new(buf));
        Ok(buf)
    }

    /// Flush all pending writes to disk.
    pub fn flush(&self) -> io::Result<()> {
        for shard in self.shards.iter() {
            shard.wake.notify_one();
        }

        loop {
            let mut all_done = true;
            for shard in self.shards.iter() {
                let wl = shard.write_list.lock().unwrap();
                let empty = wl.is_empty();
                drop(wl);
                let flushed = shard.flushed_len.load(Ordering::Relaxed);
                let allocated = shard.len.load(Ordering::Relaxed);
                if !empty || flushed < allocated {
                    all_done = false;
                    break;
                }
            }
            if all_done {
                break;
            }
            for shard in self.shards.iter() {
                shard.wake.notify_one();
            }
            thread::yield_now();
        }
        Ok(())
    }

    /// Number of unique blocks in the store.
    pub fn block_count(&self) -> u64 {
        self.shards
            .iter()
            .map(|s| s.len.load(Ordering::Relaxed) / DEDUP_BLOCK_SIZE as u64)
            .sum()
    }

    /// Total bytes on disk.
    pub fn disk_bytes(&self) -> u64 {
        self.shards
            .iter()
            .map(|s| s.len.load(Ordering::Relaxed))
            .sum()
    }

    /// Path to the dedup data directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for DedupPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        for shard in self.shards.iter() {
            shard.wake.notify_one();
        }
        let mut handles = self.flush_workers.lock().unwrap();
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }
}

/// Background flush worker.
fn flush_worker(
    shards: &[Shard],
    mut writers: Vec<(usize, File)>,
    cache: &WriteBackCache,
    stats: &DedupStats,
    shutdown: &AtomicBool,
) {
    loop {
        let mut did_work = false;

        for (shard_id, writer) in &mut writers {
            let shard = &shards[*shard_id];

            let entries: Vec<WriteEntry> = {
                let mut wl = shard.write_list.lock().unwrap();
                std::mem::take(&mut *wl)
            };

            if entries.is_empty() {
                continue;
            }
            did_work = true;

            let mut entries = entries;
            entries.sort_by_key(|e| e.offset);

            let mut i = 0;
            while i < entries.len() {
                let base_offset = entries[i].offset;
                let mut buf = Vec::with_capacity(entries.len() * DEDUP_BLOCK_SIZE);
                buf.extend_from_slice(&*entries[i].data);
                let mut j = i + 1;
                while j < entries.len()
                    && entries[j].offset
                        == base_offset + (j - i) as u64 * DEDUP_BLOCK_SIZE as u64
                {
                    buf.extend_from_slice(&*entries[j].data);
                    j += 1;
                }

                #[cfg(unix)]
                {
                    writer.write_all_at(&buf, base_offset).unwrap_or_else(|e| {
                        panic!(
                            "flush worker: pwrite to shard {} at offset {} failed: {}",
                            shard_id, base_offset, e
                        );
                    });
                }

                stats.flush_ops.fetch_add(1, Ordering::Relaxed);
                stats
                    .flush_bytes
                    .fetch_add(buf.len() as u64, Ordering::Relaxed);
                i = j;
            }

            writer.sync_data().unwrap_or_else(|e| {
                panic!("flush worker: sync_data on shard {} failed: {}", shard_id, e);
            });

            let max_end = entries.last().unwrap().offset + DEDUP_BLOCK_SIZE as u64;
            shard.flushed_len.fetch_max(max_end, Ordering::Release);

            let refs: Vec<u64> = entries.iter().map(|e| e.dedup_ref).collect();
            cache.unpin_batch(&refs);
        }

        if shutdown.load(Ordering::Relaxed) {
            let mut remaining = false;
            for (shard_id, _) in &writers {
                let wl = shards[*shard_id].write_list.lock().unwrap();
                if !wl.is_empty() {
                    remaining = true;
                    break;
                }
            }
            if !remaining {
                break;
            }
            continue;
        }

        if !did_work {
            if let Some((first_shard_id, _)) = writers.first() {
                let shard = &shards[*first_shard_id];
                let wl = shard.write_list.lock().unwrap();
                let _ = shard.wake.wait_timeout(wl, Duration::from_millis(100));
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Compute XXH3-128 hash of a block.
fn hash_block(data: &[u8; DEDUP_BLOCK_SIZE]) -> BlockHash {
    xxhash_rust::xxh3::xxh3_128(data)
}

/// Generate shard file path: `dedup_000.data`, `dedup_001.data`, etc.
fn shard_path(dir: &Path, shard_id: usize) -> PathBuf {
    dir.join(format!("dedup_{:03}.data", shard_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::tempfile::TempDir;

    fn make_pool(cache_bytes: usize) -> (DedupPool, TempDir) {
        let dir = TempDir::new().unwrap();
        let pool = DedupPool::open(dir.path(), 4, cache_bytes, 2).unwrap();
        (pool, dir)
    }

    #[test]
    fn store_and_read_block() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data = b"hello world";
        let dedup_ref = pool.store_block(data).unwrap();

        let block = pool.read_block(dedup_ref).unwrap();
        assert_eq!(&block[..data.len()], data);
        assert!(block[data.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn dedup_identical_blocks() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data = [0xABu8; 4096];

        let ref1 = pool.store_block(&data).unwrap();
        let ref2 = pool.store_block(&data).unwrap();

        assert_eq!(ref1, ref2);
        assert_eq!(pool.stats.blocks_stored.load(Ordering::Relaxed), 1);
        assert_eq!(pool.stats.blocks_deduped.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn distinct_blocks_get_different_refs() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data1 = [0x01u8; 4096];
        let data2 = [0x02u8; 4096];

        let ref1 = pool.store_block(&data1).unwrap();
        let ref2 = pool.store_block(&data2).unwrap();

        assert_ne!(ref1, ref2);
    }

    #[test]
    fn reopen_rebuilds_index() {
        let dir = TempDir::new().unwrap();
        let data = [0xCDu8; 4096];

        {
            let pool = DedupPool::open(dir.path(), 4, 64 * DEDUP_BLOCK_SIZE, 2).unwrap();
            pool.store_block(&data).unwrap();
            pool.flush().unwrap();
        }

        {
            let pool = DedupPool::open(dir.path(), 4, 64 * DEDUP_BLOCK_SIZE, 2).unwrap();
            let dedup_ref = pool.store_block(&data).unwrap();
            assert_eq!(pool.stats.blocks_deduped.load(Ordering::Relaxed), 1);

            pool.flush().unwrap();
            let block = pool.read_block(dedup_ref).unwrap();
            assert_eq!(block, data);
        }
    }

    #[test]
    fn read_cache_hit() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data = [0xEFu8; 4096];

        let dedup_ref = pool.store_block(&data).unwrap();

        let _block = pool.read_block(dedup_ref).unwrap();
        assert_eq!(pool.stats.cache_hits.load(Ordering::Relaxed), 1);

        let _block = pool.read_block(dedup_ref).unwrap();
        assert_eq!(pool.stats.cache_hits.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn partial_block_zero_padded() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data = b"short";

        let dedup_ref = pool.store_block(data).unwrap();
        let block = pool.read_block(dedup_ref).unwrap();

        assert_eq!(&block[..5], b"short");
        assert!(block[5..].iter().all(|&b| b == 0));
    }

    #[test]
    fn flush_persists_data() {
        let (pool, _dir) = make_pool(64 * DEDUP_BLOCK_SIZE);
        let data = [0x99u8; 4096];
        let dedup_ref = pool.store_block(&data).unwrap();
        pool.flush().unwrap();

        let block = pool.read_block(dedup_ref).unwrap();
        assert_eq!(block, data);
    }
}
