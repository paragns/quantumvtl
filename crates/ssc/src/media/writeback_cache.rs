//! Write-back cache for dedup blocks — decouples write ACKs from disk I/O.
//!
//! Blocks enter the cache as "pinned" (dirty, not yet flushed to disk).
//! Flush workers unpin blocks after writing them to shard files.
//! Unpinned blocks remain cached for reads and are evicted LRU when space
//! is needed. If the cache is full and all entries are pinned, writers
//! block on a condvar (backpressure).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};

use indexmap::IndexMap;

use super::dedup::DEDUP_BLOCK_SIZE;

/// Cache entry: a 4KB block with a pinned (dirty) flag.
struct CacheEntry {
    data: Box<[u8; DEDUP_BLOCK_SIZE]>,
    pinned: bool,
}

/// Guarded interior of the cache.
struct CacheInner {
    /// Insertion-ordered map: dedup_ref → entry.
    /// New entries go to the back; on `get()` we move to back (MRU).
    entries: IndexMap<u64, CacheEntry>,
    capacity: usize,
    pinned_count: usize,
}

/// Statistics (atomic, lockfree reads).
#[derive(Debug, Default)]
pub struct CacheStats {
    pub writer_stalls: AtomicU64,
}

/// A write-back block cache with pinned/unpinned semantics.
pub struct WriteBackCache {
    inner: Mutex<CacheInner>,
    /// Signaled when blocks are unpinned or evicted (frees space for writers).
    space_available: Condvar,
    pub stats: CacheStats,
}

impl WriteBackCache {
    /// Create a new cache that holds up to `capacity_blocks` blocks.
    pub fn new(capacity_blocks: usize) -> Self {
        assert!(capacity_blocks > 0, "cache capacity must be > 0");
        Self {
            inner: Mutex::new(CacheInner {
                entries: IndexMap::with_capacity(capacity_blocks),
                capacity: capacity_blocks,
                pinned_count: 0,
            }),
            space_available: Condvar::new(),
            stats: CacheStats::default(),
        }
    }

    /// Insert a dirty (pinned) block. If the cache is full, evict LRU unpinned.
    /// If all entries are pinned, block the caller until space is available.
    pub fn insert_pinned(&self, dedup_ref: u64, data: Box<[u8; DEDUP_BLOCK_SIZE]>) {
        let mut inner = self.inner.lock().unwrap();
        loop {
            // If already present: remove old, reinsert at back as pinned.
            if let Some(old) = inner.entries.swap_remove(&dedup_ref) {
                if old.pinned {
                    inner.pinned_count -= 1;
                }
                inner.entries.insert(dedup_ref, CacheEntry { data, pinned: true });
                inner.pinned_count += 1;
                return;
            }

            if inner.entries.len() < inner.capacity {
                inner.entries.insert(dedup_ref, CacheEntry { data, pinned: true });
                inner.pinned_count += 1;
                return;
            }

            // Cache full — try to evict LRU unpinned entry.
            if inner.pinned_count < inner.entries.len() {
                let evict_idx = inner
                    .entries
                    .iter()
                    .position(|(_, e)| !e.pinned)
                    .expect("pinned_count < len but no unpinned found");
                inner.entries.swap_remove_index(evict_idx);
                inner.entries.insert(dedup_ref, CacheEntry { data, pinned: true });
                inner.pinned_count += 1;
                return;
            }

            // All entries pinned — backpressure: block until space freed.
            self.stats.writer_stalls.fetch_add(1, Ordering::Relaxed);
            inner = self.space_available.wait(inner).unwrap();
        }
    }

    /// Mark a batch of blocks as unpinned (clean) after flush.
    pub fn unpin_batch(&self, refs: &[u64]) {
        let mut inner = self.inner.lock().unwrap();
        for &r in refs {
            if let Some(entry) = inner.entries.get_mut(&r) {
                if entry.pinned {
                    entry.pinned = false;
                    inner.pinned_count -= 1;
                }
            }
        }
        self.space_available.notify_all();
    }

    /// Read a block from the cache.
    pub fn get(&self, dedup_ref: u64) -> Option<[u8; DEDUP_BLOCK_SIZE]> {
        let inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.entries.get(&dedup_ref) {
            Some(*entry.data)
        } else {
            None
        }
    }

    /// Insert a clean (unpinned) block — e.g. on a read miss from disk.
    pub fn insert_clean(&self, dedup_ref: u64, data: Box<[u8; DEDUP_BLOCK_SIZE]>) {
        let mut inner = self.inner.lock().unwrap();

        // If already present, update data (remove + reinsert at back).
        if let Some(old) = inner.entries.swap_remove(&dedup_ref) {
            if old.pinned {
                inner.pinned_count -= 1;
            }
            let pinned = old.pinned;
            inner.entries.insert(dedup_ref, CacheEntry { data, pinned });
            if pinned {
                inner.pinned_count += 1;
            }
            return;
        }

        if inner.entries.len() < inner.capacity {
            inner.entries.insert(dedup_ref, CacheEntry { data, pinned: false });
            return;
        }

        // Evict LRU unpinned. If all pinned, skip.
        if inner.pinned_count < inner.entries.len() {
            let evict_idx = inner
                .entries
                .iter()
                .position(|(_, e)| !e.pinned)
                .expect("pinned_count < len but no unpinned found");
            inner.entries.swap_remove_index(evict_idx);
            inner.entries.insert(dedup_ref, CacheEntry { data, pinned: false });
        }
    }

    /// Return the current number of pinned (dirty) blocks.
    pub fn pinned_count(&self) -> usize {
        self.inner.lock().unwrap().pinned_count
    }

    /// Return the total number of cached blocks.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().entries.len()
    }
}
