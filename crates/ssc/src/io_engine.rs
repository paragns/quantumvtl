//! Dedicated I/O thread for tape store operations.
//!
//! Decouples file I/O syscalls from the SCSI command path, isolating
//! seek/write/fsync jitter from the buffer simulation timing. Provides
//! write batching: one seek + N writes + one fsync per batch.

use std::io;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};

use crate::media::mam::MamAttributes;
use crate::media::store::TapeStore;
use crate::media::tape::{RecordDescriptor, TapeMedia, TapePartition};

/// A single write to be batched.
pub struct IoWrite {
    pub data: Vec<u8>,
    pub native_length: u32,
    pub is_compressed: bool,
    pub partition: u32,
    pub record_num: u64,
}

/// Result of a single write operation.
pub struct WriteResult {
    pub descriptor: RecordDescriptor,
    pub native_bytes: u64,
    pub on_disk_bytes: u64,
}

/// Commands sent to the I/O thread.
pub enum IoCommand {
    /// Write a batch of records. Returns descriptors for each write.
    WriteBatch {
        writes: Vec<IoWrite>,
        reply: SyncSender<io::Result<Vec<WriteResult>>>,
    },
    /// Read data from the store at the given offset and length.
    Read {
        offset: u64,
        length: u32,
        reply: SyncSender<io::Result<Vec<u8>>>,
    },
    /// Flush the data file to disk.
    Flush {
        reply: SyncSender<io::Result<()>>,
    },
    /// Truncate the data file to a new length.
    Truncate {
        new_len: u64,
        reply: SyncSender<io::Result<()>>,
    },
    /// Save a record descriptor to the redb index.
    SaveRecord {
        partition: u32,
        record_num: u64,
        desc: RecordDescriptor,
        reply: SyncSender<io::Result<()>>,
    },
    /// Remove all records from a partition starting at the given index.
    RemoveRecordsFrom {
        partition: u32,
        from: u64,
        reply: SyncSender<io::Result<()>>,
    },
    /// Save media metadata to the redb store.
    SaveMediaMeta {
        media: Box<TapeMedia>,
        reply: SyncSender<io::Result<()>>,
    },
    /// Save partition stats to the redb store.
    SavePartitionStats {
        idx: u32,
        partition: Box<TapePartition>,
        reply: SyncSender<io::Result<()>>,
    },
    /// Save MAM attributes to the redb store.
    SaveMam {
        mam: Box<MamAttributes>,
        reply: SyncSender<io::Result<()>>,
    },
    /// Clear all records for a partition.
    ClearPartitionRecords {
        partition: u32,
        reply: SyncSender<io::Result<()>>,
    },
    /// Clear all records for all partitions.
    ClearAllRecords {
        reply: SyncSender<io::Result<()>>,
    },
    /// Clear all partition stats.
    ClearAllPartitionStats {
        reply: SyncSender<io::Result<()>>,
    },
    /// Shut down the I/O thread.
    Shutdown,
}

/// Handle to communicate with the I/O thread.
pub struct IoHandle {
    tx: SyncSender<IoCommand>,
    join_handle: Option<JoinHandle<()>>,
}

impl IoHandle {
    /// Spawn a new I/O thread owning the given TapeStore.
    pub fn spawn(store: TapeStore) -> Self {
        let (tx, rx) = mpsc::sync_channel::<IoCommand>(64);
        let join_handle = thread::Builder::new()
            .name("tape-io".into())
            .spawn(move || {
                io_thread_main(store, rx);
            })
            .expect("failed to spawn I/O thread");

        Self {
            tx,
            join_handle: Some(join_handle),
        }
    }

    /// Submit a batch of writes. Returns results synchronously.
    pub fn write_batch(&self, writes: Vec<IoWrite>) -> io::Result<Vec<WriteResult>> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::WriteBatch {
                writes,
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Read data from the store (blocking).
    pub fn read_sync(&self, offset: u64, length: u32) -> io::Result<Vec<u8>> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::Read {
                offset,
                length,
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Flush the data file to disk (blocking).
    pub fn flush_sync(&self) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::Flush { reply: reply_tx })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Truncate the data file (blocking).
    pub fn truncate_sync(&self, new_len: u64) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::Truncate {
                new_len,
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Save a record descriptor to the redb index (blocking).
    pub fn save_record_sync(
        &self,
        partition: u32,
        record_num: u64,
        desc: &RecordDescriptor,
    ) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::SaveRecord {
                partition,
                record_num,
                desc: desc.clone(),
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Remove records from a partition starting at the given index (blocking).
    pub fn remove_records_from_sync(&self, partition: u32, from: u64) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::RemoveRecordsFrom {
                partition,
                from,
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Save media metadata (blocking).
    pub fn save_media_meta_sync(&self, media: &TapeMedia) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::SaveMediaMeta {
                media: Box::new(media.clone()),
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Save partition stats (blocking).
    pub fn save_partition_stats_sync(&self, idx: u32, partition: &TapePartition) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::SavePartitionStats {
                idx,
                partition: Box::new(partition.clone()),
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Clear all records for a partition (blocking).
    pub fn clear_partition_records_sync(&self, partition: u32) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::ClearPartitionRecords {
                partition,
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Clear all records for all partitions (blocking).
    pub fn clear_all_records_sync(&self) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::ClearAllRecords { reply: reply_tx })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Clear all partition stats (blocking).
    pub fn clear_all_partition_stats_sync(&self) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::ClearAllPartitionStats { reply: reply_tx })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Save MAM attributes (blocking).
    pub fn save_mam_sync(&self, mam: &MamAttributes) -> io::Result<()> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(IoCommand::SaveMam {
                mam: Box::new(mam.clone()),
                reply: reply_tx,
            })
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?;
        reply_rx
            .recv()
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "I/O thread gone"))?
    }

    /// Shut down the I/O thread and wait for it to finish.
    pub fn shutdown(mut self) {
        let _ = self.tx.send(IoCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for IoHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(IoCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Main loop of the I/O thread.
fn io_thread_main(mut store: TapeStore, rx: Receiver<IoCommand>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            IoCommand::WriteBatch { writes, reply } => {
                let result = execute_write_batch(&mut store, writes);
                let _ = reply.send(result);
            }
            IoCommand::Read {
                offset,
                length,
                reply,
            } => {
                let result = store.read_data(offset, length);
                let _ = reply.send(result);
            }
            IoCommand::Flush { reply } => {
                let _ = reply.send(store.sync_data());
            }
            IoCommand::Truncate { new_len, reply } => {
                let result = store.truncate_data(new_len);
                let _ = reply.send(result);
            }
            IoCommand::SaveRecord {
                partition,
                record_num,
                desc,
                reply,
            } => {
                let result = store.save_record(partition, record_num, &desc);
                let _ = reply.send(result);
            }
            IoCommand::RemoveRecordsFrom {
                partition,
                from,
                reply,
            } => {
                let result = store.remove_records_from(partition, from);
                let _ = reply.send(result);
            }
            IoCommand::SaveMediaMeta { media, reply } => {
                let result = store.save_media_meta(&media);
                let _ = reply.send(result);
            }
            IoCommand::SavePartitionStats { idx, partition, reply } => {
                let result = store.save_partition_stats(idx, &partition);
                let _ = reply.send(result);
            }
            IoCommand::SaveMam { mam, reply } => {
                let result = store.save_mam(&mam);
                let _ = reply.send(result);
            }
            IoCommand::ClearPartitionRecords { partition, reply } => {
                let result = store.clear_partition_records(partition);
                let _ = reply.send(result);
            }
            IoCommand::ClearAllRecords { reply } => {
                let result = store.clear_all_records();
                let _ = reply.send(result);
            }
            IoCommand::ClearAllPartitionStats { reply } => {
                let result = store.clear_all_partition_stats();
                let _ = reply.send(result);
            }
            IoCommand::Shutdown => break,
        }
    }
}

/// Execute a batch of writes: append all data, then save all records.
fn execute_write_batch(
    store: &mut TapeStore,
    writes: Vec<IoWrite>,
) -> io::Result<Vec<WriteResult>> {
    let mut results = Vec::with_capacity(writes.len());

    for w in writes {
        let (offset, length) = store.append_data(&w.data)?;

        let descriptor = if w.is_compressed {
            RecordDescriptor::CompressedData {
                offset,
                compressed_length: length,
                native_length: w.native_length,
            }
        } else {
            RecordDescriptor::Data { offset, length }
        };

        let on_disk_bytes = length as u64;
        let native_bytes = w.native_length as u64;

        store.save_record(w.partition, w.record_num, &descriptor)?;

        results.push(WriteResult {
            descriptor,
            native_bytes,
            on_disk_bytes,
        });
    }

    Ok(results)
}
