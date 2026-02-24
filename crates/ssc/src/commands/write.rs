//! WRITE(6) command handler.

use crate::media::tape::{DriveMediaState, RecordDescriptor};
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::{info, warn};

/// Compress and write a single block to the store.
///
/// Matches real LTO drive behavior: when DCE=1, every block is compressed
/// and stored as `CompressedData` regardless of whether compression reduced
/// the size. When DCE=0, blocks are stored as raw `Data`.
///
/// Returns `(descriptor, native_bytes, on_disk_bytes)`.
fn write_block(
    store: &mut crate::media::store::TapeStore,
    block: &[u8],
    compress: bool,
) -> Result<(RecordDescriptor, u64, u64), ScsiResult> {
    let native_len = block.len() as u32;

    if compress {
        // Real LTO drives always run the compression engine when DCE=1.
        let compressed = zstd::encode_all(block, -3).map_err(|e| {
            warn!(error = %e, "zstd compression failed");
            SenseBuilder::medium_error().to_check_condition()
        })?;
        let compressed_length = compressed.len() as u32;
        let (offset, _) = store.append_data(&compressed).map_err(|e| {
            warn!(error = %e, "failed to append compressed data to tape store");
            SenseBuilder::medium_error().to_check_condition()
        })?;
        let desc = RecordDescriptor::CompressedData {
            offset,
            compressed_length,
            native_length: native_len,
        };
        Ok((desc, native_len as u64, compressed_length as u64))
    } else {
        let (offset, length) = store.append_data(block).map_err(|e| {
            warn!(error = %e, "failed to append data to tape store");
            SenseBuilder::medium_error().to_check_condition()
        })?;
        let desc = RecordDescriptor::Data { offset, length };
        Ok((desc, native_len as u64, native_len as u64))
    }
}

/// Handle WRITE(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length.
pub fn handle_write_6(
    cdb: &[u8],
    data_out: &[u8],
    media_state: &mut DriveMediaState,
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
    let pos = media_state.position.block_number as usize;
    let partition_idx = media_state.position.partition as u32;

    // Truncate from current position (overwrite mode)
    truncate_at_position(media_state, pos, partition_idx);

    if fixed {
        // Fixed-block mode: data_out contains transfer_length blocks
        if transfer_length > 0 {
            let block_size = data_out.len() / transfer_length as usize;
            for i in 0..transfer_length as usize {
                let start = i * block_size;
                let end = start + block_size;
                let block = &data_out[start..end];

                let (desc, native_bytes, on_disk_bytes) =
                    match write_block(&mut media_state.store, block, compression_enabled) {
                        Ok(r) => r,
                        Err(scsi_err) => return scsi_err,
                    };

                let record_num = media_state.current_partition().records.len() as u64;
                if let Err(e) = media_state
                    .store
                    .save_record(partition_idx, record_num, &desc)
                {
                    warn!(error = %e, "failed to save record index");
                    return SenseBuilder::medium_error().to_check_condition();
                }

                let partition = media_state.current_partition_mut();
                partition.records.push(desc);
                partition.bytes_written_native += native_bytes;
                partition.bytes_written_compressed += on_disk_bytes;
            }
            media_state.position.block_number += transfer_length as u64;
        }
    } else {
        // Variable-block mode: data_out is one block
        let (desc, native_bytes, on_disk_bytes) =
            match write_block(&mut media_state.store, data_out, compression_enabled) {
                Ok(r) => r,
                Err(scsi_err) => return scsi_err,
            };

        let record_num = media_state.current_partition().records.len() as u64;
        if let Err(e) = media_state
            .store
            .save_record(partition_idx, record_num, &desc)
        {
            warn!(error = %e, "failed to save record index");
            return SenseBuilder::medium_error().to_check_condition();
        }

        let partition = media_state.current_partition_mut();
        partition.records.push(desc);
        partition.bytes_written_native += native_bytes;
        partition.bytes_written_compressed += on_disk_bytes;
        media_state.position.block_number += 1;
    }

    info!(
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
        if let Err(e) = media_state
            .store
            .remove_records_from(partition_idx, pos as u64)
        {
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
                        RecordDescriptor::Filemark => {}
                    }
                }
                max_end
            } else {
                0
            };
            if let Err(e) = media_state.store.truncate_data(data_truncate_len) {
                warn!(error = %e, "failed to truncate data file");
            }
        }
    }
}
