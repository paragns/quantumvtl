//! WRITE(6) command handler.

use crate::buffer::DriveBuffer;
use crate::media::tape::{DriveMediaState, RecordDescriptor};
use crate::sense::{self, SenseBuilder};
use iscsi_target::SimulationClock;
use crate::ScsiResult;
use tracing::{trace, warn};

/// Compress and write a single block to the store via the I/O handle.
///
/// Matches real LTO drive behavior: when DCE=1, every block is compressed
/// and stored as `CompressedData` regardless of whether compression reduced
/// the size. When DCE=0, blocks are stored as raw `Data`.
///
/// When `dedup` is true, compression is skipped and the block is sent to the
/// dedup store for block-level deduplication.
///
/// Returns `(descriptor, native_bytes, on_disk_bytes)`.
fn write_block(
    io_handle: &crate::io_engine::IoHandle,
    block: &[u8],
    compress: bool,
    dedup: bool,
    partition: u32,
    record_num: u64,
) -> Result<(RecordDescriptor, u64, u64), ScsiResult> {
    let native_len = block.len() as u32;

    let (data, is_compressed, is_dedup) = if dedup {
        // Dedup: send raw data, no compression (dedup on compressed data is ineffective)
        (block.to_vec(), false, true)
    } else if compress {
        let compressed = zstd::encode_all(block, -3).map_err(|e| {
            warn!(error = %e, "zstd compression failed");
            SenseBuilder::medium_error().to_check_condition()
        })?;
        (compressed, true, false)
    } else {
        (block.to_vec(), false, false)
    };

    let writes = vec![crate::io_engine::IoWrite {
        data,
        native_length: native_len,
        is_compressed,
        is_dedup,
        partition,
        record_num,
    }];

    let mut results = io_handle.write_batch(writes).map_err(|e| {
        warn!(error = %e, "failed to write block to tape store");
        SenseBuilder::medium_error().to_check_condition()
    })?;

    let r = results.remove(0);
    Ok((r.descriptor, r.native_bytes, r.on_disk_bytes))
}

/// Handle WRITE(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length.
pub fn handle_write_6(
    cdb: &[u8],
    data_out: &[u8],
    media_state: &mut DriveMediaState,
    buffer: &mut Option<DriveBuffer>,
    clock: &SimulationClock,
) -> ScsiResult {
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    if transfer_length == 0 {
        return sense::good();
    }

    // Check write protection
    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    let compression_enabled = media_state.media.compression_enabled;
    let dedup_enabled = media_state.dedup_store.is_some();
    let pos = media_state.position.block_number as usize;
    let partition_idx = media_state.position.partition as u32;

    // Truncate from current position (overwrite mode)
    truncate_at_position(media_state, pos, partition_idx);

    // Pre-compute per-partition capacity limit for overflow detection.
    let num_partitions = media_state.media.partitions.len() as u64;
    let partition_capacity = media_state.media.native_capacity_bytes() / num_partitions.max(1);

    if fixed {
        // Fixed-block mode: data_out contains transfer_length blocks
        if transfer_length > 0 {
            let block_size = data_out.len() / transfer_length as usize;
            for i in 0..transfer_length as usize {
                // Check capacity before writing this block
                let partition = media_state.current_partition();
                if partition.bytes_written_native + block_size as u64 > partition_capacity {
                    warn!(
                        partition = media_state.position.partition,
                        written = partition.bytes_written_native,
                        capacity = partition_capacity,
                        "partition capacity exceeded, returning VOLUME OVERFLOW"
                    );
                    return SenseBuilder::volume_overflow().to_check_condition();
                }

                let start = i * block_size;
                let end = start + block_size;
                let block = &data_out[start..end];

                let record_num = media_state.current_partition().records.len() as u64;
                let (desc, native_bytes, on_disk_bytes) =
                    match write_block(&media_state.io_handle, block, compression_enabled, dedup_enabled, partition_idx, record_num) {
                        Ok(r) => r,
                        Err(scsi_err) => return scsi_err,
                    };

                let partition = media_state.current_partition_mut();
                partition.records.push(desc);
                partition.bytes_written_native += native_bytes;
                partition.bytes_written_compressed += on_disk_bytes;

                // Buffer simulation: account for this write
                if let Some(ref mut buf) = buffer {
                    let stall = buf.accept_write(native_bytes as usize, clock);
                    if !stall.is_zero() {
                        clock.sleep_sync(stall);
                        buf.tick(clock);
                    }
                }
            }
            media_state.position.block_number += transfer_length as u64;
        }
    } else {
        // Variable-block mode: data_out is one block

        // Check capacity before writing
        let partition = media_state.current_partition();
        if partition.bytes_written_native + data_out.len() as u64 > partition_capacity {
            warn!(
                partition = media_state.position.partition,
                written = partition.bytes_written_native,
                capacity = partition_capacity,
                "partition capacity exceeded, returning VOLUME OVERFLOW"
            );
            return SenseBuilder::volume_overflow().to_check_condition();
        }

        let record_num = media_state.current_partition().records.len() as u64;
        let (desc, native_bytes, on_disk_bytes) =
            match write_block(&media_state.io_handle, data_out, compression_enabled, dedup_enabled, partition_idx, record_num) {
                Ok(r) => r,
                Err(scsi_err) => return scsi_err,
            };

        let partition = media_state.current_partition_mut();
        partition.records.push(desc);
        partition.bytes_written_native += native_bytes;
        partition.bytes_written_compressed += on_disk_bytes;
        media_state.position.block_number += 1;

        // Buffer simulation: account for this write
        if let Some(ref mut buf) = buffer {
            let stall = buf.accept_write(native_bytes as usize, clock);
            if !stall.is_zero() {
                clock.sleep_sync(stall);
                buf.tick(clock);
            }
        }
    }

    trace!(
        partition = media_state.position.partition,
        position = media_state.position.block_number,
        total_records = media_state.current_partition().records.len(),
        "WRITE complete"
    );

    sense::good()
}

/// Truncate records at the given position and clean up the store.
fn truncate_at_position(media_state: &mut DriveMediaState, pos: usize, partition_idx: u32) {
    let num_partitions = media_state.media.partitions.len();
    let partition = media_state.current_partition_mut();
    let old_len = partition.records.len();
    if pos < old_len {
        partition.records.truncate(pos);
        partition.rebuild_filemark_index();

        // Remove redb record entries from the truncation point onward
        if let Err(e) = media_state.io_handle.remove_records_from_sync(partition_idx, pos as u64) {
            warn!(error = %e, "failed to remove truncated records from store");
        }

        // Only truncate the data file for single-partition tapes. With multiple
        // partitions the data file is shared, so truncating based on one partition's
        // records would destroy data belonging to other partitions.
        if num_partitions <= 1 {
            let data_truncate_len = if pos > 0 {
                let mut max_end: u64 = 0;
                for rec in &media_state.current_partition().records[..pos] {
                    match rec {
                        RecordDescriptor::Data { offset, length } => {
                            let end = offset + *length as u64;
                            if end > max_end {
                                max_end = end;
                            }
                        }
                        RecordDescriptor::CompressedData {
                            offset,
                            compressed_length,
                            ..
                        } => {
                            let end = offset + *compressed_length as u64;
                            if end > max_end {
                                max_end = end;
                            }
                        }
                        RecordDescriptor::DedupData {
                            offsets_offset,
                            num_chunks,
                            ..
                        } => {
                            let end = offsets_offset + (*num_chunks as u64) * 8;
                            if end > max_end {
                                max_end = end;
                            }
                        }
                        RecordDescriptor::Filemark => {}
                    }
                }
                max_end
            } else {
                0
            };
            if let Err(e) = media_state.io_handle.truncate_sync(data_truncate_len) {
                warn!(error = %e, "failed to truncate data file");
            }
        }
    }
}
