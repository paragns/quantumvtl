//! Deduplication store — shared block-level dedup across all tapes.
//!
//! Unique 4KB blocks are stored in a single append-only `dedup.data` file.
//! An in-memory hash map provides duplicate detection on writes; an LRU cache
//! avoids small random reads on the read path.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

use dashmap::DashMap;
use lru::LruCache;
use tracing::{info, warn};

/// Block size for dedup (4KB).
pub const DEDUP_BLOCK_SIZE: usize = 4096;

/// 128-bit hash of a dedup block.
pub type BlockHash = [u8; 16];

/// Statistics for the dedup store.
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
}

/// Shared dedup store: append-only file + in-memory hash index + LRU read cache.
pub struct DedupStore {
    /// Hash → offset for dedup detection (write path).
    index: DashMap<BlockHash, u64>,
    /// Offset → block data LRU cache (read path).
    cache: Mutex<LruCache<u64, Box<[u8; DEDUP_BLOCK_SIZE]>>>,
    /// Append-only data file (write path).
    file: Mutex<File>,
    /// Current end-of-file offset (next append position).
    len: AtomicU64,
    /// Read-only file handle for concurrent pread (no seek contention).
    read_file: File,
    /// Path to the dedup data file.
    path: PathBuf,
    /// Statistics.
    pub stats: DedupStats,
}

impl DedupStore {
    /// Open or create a dedup store. Rebuilds the in-memory index by scanning the file.
    pub fn open(data_dir: &Path, cache_blocks: usize) -> io::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let path = data_dir.join("dedup.data");

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;

        let read_file = File::open(&path)?;

        let file_len = file.metadata()?.len();

        // Validate file length is block-aligned
        if file_len % DEDUP_BLOCK_SIZE as u64 != 0 {
            warn!(
                file_len,
                "dedup.data has non-aligned length, truncating to block boundary"
            );
            let aligned = file_len / DEDUP_BLOCK_SIZE as u64 * DEDUP_BLOCK_SIZE as u64;
            file.set_len(aligned)?;
        }

        let aligned_len = file_len / DEDUP_BLOCK_SIZE as u64 * DEDUP_BLOCK_SIZE as u64;
        let num_blocks = aligned_len / DEDUP_BLOCK_SIZE as u64;

        let index: DashMap<BlockHash, u64> = DashMap::with_capacity(num_blocks as usize);

        // Rebuild index by scanning existing blocks
        if num_blocks > 0 {
            info!(
                num_blocks,
                file_bytes = aligned_len,
                "rebuilding dedup index from existing data"
            );

            let mut scan_file = File::open(&path)?;
            let mut buf = [0u8; DEDUP_BLOCK_SIZE];

            for block_num in 0..num_blocks {
                let offset = block_num * DEDUP_BLOCK_SIZE as u64;
                scan_file.read_exact(&mut buf).map_err(|e| {
                    io::Error::new(
                        e.kind(),
                        format!("failed to read block {} at offset {}: {}", block_num, offset, e),
                    )
                })?;
                let hash = hash_block(&buf);
                index.insert(hash, offset);
            }

            info!(
                unique_blocks = index.len(),
                total_blocks = num_blocks,
                "dedup index rebuild complete"
            );
        }

        let cap = std::num::NonZeroUsize::new(cache_blocks.max(1)).unwrap();

        Ok(Self {
            index,
            cache: Mutex::new(LruCache::new(cap)),
            file: Mutex::new(file),
            len: AtomicU64::new(aligned_len),
            read_file,
            path,
            stats: DedupStats::default(),
        })
    }

    /// Store a block (≤ 4KB, zero-padded to 4KB). Returns the offset in `dedup.data`.
    /// If the block already exists, returns the existing offset (no write).
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

        // Block is new — append to file
        let mut file = self.file.lock().unwrap();

        // Double-check after acquiring write lock (another thread may have inserted)
        if let Some(existing) = self.index.get(&hash) {
            self.stats.blocks_deduped.fetch_add(1, Ordering::Relaxed);
            return Ok(*existing);
        }

        let offset = self.len.load(Ordering::Relaxed);
        file.write_all(&padded)?;
        self.len.store(offset + DEDUP_BLOCK_SIZE as u64, Ordering::Relaxed);

        // Insert into index
        self.index.insert(hash, offset);

        // Insert into cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(offset, Box::new(padded));
        }

        self.stats.blocks_stored.fetch_add(1, Ordering::Relaxed);

        Ok(offset)
    }

    /// Read a block at the given offset. Checks LRU cache first.
    pub fn read_block(&self, offset: u64) -> io::Result<[u8; DEDUP_BLOCK_SIZE]> {
        // Check cache
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(block) = cache.get(&offset) {
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(**block);
            }
        }

        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Read from file using pread (no seek contention)
        let mut buf = [0u8; DEDUP_BLOCK_SIZE];
        #[cfg(unix)]
        {
            self.read_file.read_at(&mut buf, offset)?;
        }
        #[cfg(not(unix))]
        {
            // Fallback: use the write file with lock (less efficient)
            use std::io::Seek;
            let mut file = self.file.lock().unwrap();
            file.seek(std::io::SeekFrom::Start(offset))?;
            file.read_exact(&mut buf)?;
        }

        // Insert into cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(offset, Box::new(buf));
        }

        Ok(buf)
    }

    /// Number of unique blocks in the store.
    pub fn block_count(&self) -> u64 {
        self.len.load(Ordering::Relaxed) / DEDUP_BLOCK_SIZE as u64
    }

    /// Total bytes on disk.
    pub fn disk_bytes(&self) -> u64 {
        self.len.load(Ordering::Relaxed)
    }

    /// Path to the dedup data file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Compute XXH3-128 hash of a block.
fn hash_block(data: &[u8; DEDUP_BLOCK_SIZE]) -> BlockHash {
    let h = xxhash_rust::xxh3::xxh3_128(data);
    h.to_le_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::tempfile::TempDir;

    fn make_store(cache_blocks: usize) -> (DedupStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = DedupStore::open(dir.path(), cache_blocks).unwrap();
        (store, dir)
    }

    #[test]
    fn store_and_read_block() {
        let (store, _dir) = make_store(16);
        let data = b"hello world";
        let offset = store.store_block(data).unwrap();
        assert_eq!(offset, 0);

        let block = store.read_block(offset).unwrap();
        // First 11 bytes should match, rest should be zero-padded
        assert_eq!(&block[..data.len()], data);
        assert!(block[data.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn dedup_identical_blocks() {
        let (store, _dir) = make_store(16);
        let data = [0xABu8; 4096];

        let offset1 = store.store_block(&data).unwrap();
        let offset2 = store.store_block(&data).unwrap();

        // Same block should return same offset
        assert_eq!(offset1, offset2);
        // Only one block on disk
        assert_eq!(store.block_count(), 1);
        assert_eq!(store.stats.blocks_stored.load(Ordering::Relaxed), 1);
        assert_eq!(store.stats.blocks_deduped.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn distinct_blocks_get_different_offsets() {
        let (store, _dir) = make_store(16);

        let data1 = [0x01u8; 4096];
        let data2 = [0x02u8; 4096];

        let offset1 = store.store_block(&data1).unwrap();
        let offset2 = store.store_block(&data2).unwrap();

        assert_ne!(offset1, offset2);
        assert_eq!(store.block_count(), 2);
    }

    #[test]
    fn reopen_rebuilds_index() {
        let dir = TempDir::new().unwrap();

        let data = [0xCDu8; 4096];

        // Store a block
        {
            let store = DedupStore::open(dir.path(), 16).unwrap();
            store.store_block(&data).unwrap();
            assert_eq!(store.block_count(), 1);
        }

        // Reopen — index should be rebuilt
        {
            let store = DedupStore::open(dir.path(), 16).unwrap();
            assert_eq!(store.block_count(), 1);

            // Storing the same block should dedup
            let offset = store.store_block(&data).unwrap();
            assert_eq!(offset, 0);
            assert_eq!(store.block_count(), 1);
            assert_eq!(store.stats.blocks_deduped.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn read_cache_hit() {
        let (store, _dir) = make_store(16);
        let data = [0xEFu8; 4096];

        let offset = store.store_block(&data).unwrap();

        // First read: was inserted into cache during store_block
        let _block = store.read_block(offset).unwrap();
        assert_eq!(store.stats.cache_hits.load(Ordering::Relaxed), 1);

        // Second read: should also hit cache
        let _block = store.read_block(offset).unwrap();
        assert_eq!(store.stats.cache_hits.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn partial_block_zero_padded() {
        let (store, _dir) = make_store(16);
        let data = b"short";

        let offset = store.store_block(data).unwrap();
        let block = store.read_block(offset).unwrap();

        assert_eq!(&block[..5], b"short");
        assert!(block[5..].iter().all(|&b| b == 0));
    }
}
