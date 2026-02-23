//! WRITE(6) command handler.

use crate::media::tape::{DriveMediaState, RecordDescriptor};
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::{trace, warn};

/// Handle WRITE(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length.
pub fn handle_write_6(cdb: &[u8], data_out: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    if transfer_length == 0 {
        return sense::good();
    }

    // Check write protection
    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    // Capture compression settings before borrowing partition mutably
    let compression_enabled = media_state.media.compression_enabled;
    let compression_ratio = media_state.media.compression_ratio;

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
                let byte_len = block.len() as u64;

                // Append data to store and get descriptor
                let desc = match media_state.store.append_data(block) {
                    Ok((offset, length)) => RecordDescriptor::Data { offset, length },
                    Err(e) => {
                        warn!(error = %e, "failed to append data to tape store");
                        return SenseBuilder::medium_error().to_check_condition();
                    }
                };

                let record_num = media_state.current_partition().records.len() as u64;
                if let Err(e) = media_state.store.save_record(partition_idx, record_num, &desc) {
                    warn!(error = %e, "failed to save record index");
                    return SenseBuilder::medium_error().to_check_condition();
                }

                let partition = media_state.current_partition_mut();
                partition.records.push(desc);
                partition.bytes_written_native += byte_len;
                if compression_enabled && compression_ratio > 0.0 {
                    partition.bytes_written_compressed += (byte_len as f64 / compression_ratio) as u64;
                } else {
                    partition.bytes_written_compressed += byte_len;
                }
            }
            media_state.position.block_number += transfer_length as u64;
        }
    } else {
        // Variable-block mode: data_out is one block
        let byte_len = data_out.len() as u64;

        let desc = match media_state.store.append_data(data_out) {
            Ok((offset, length)) => RecordDescriptor::Data { offset, length },
            Err(e) => {
                warn!(error = %e, "failed to append data to tape store");
                return SenseBuilder::medium_error().to_check_condition();
            }
        };

        let record_num = media_state.current_partition().records.len() as u64;
        if let Err(e) = media_state.store.save_record(partition_idx, record_num, &desc) {
            warn!(error = %e, "failed to save record index");
            return SenseBuilder::medium_error().to_check_condition();
        }

        let partition = media_state.current_partition_mut();
        partition.records.push(desc);
        partition.bytes_written_native += byte_len;
        if compression_enabled && compression_ratio > 0.0 {
            partition.bytes_written_compressed += (byte_len as f64 / compression_ratio) as u64;
        } else {
            partition.bytes_written_compressed += byte_len;
        }
        media_state.position.block_number += 1;
    }

    trace!(
        position = media_state.position.block_number,
        total_records = media_state.current_partition().records.len(),
        "WRITE complete"
    );

    sense::good()
}

/// Truncate records at the given position and clean up the store.
fn truncate_at_position(media_state: &mut DriveMediaState, pos: usize, partition_idx: u32) {
    let partition = media_state.current_partition_mut();
    let old_len = partition.records.len();
    if pos < old_len {
        // Find the data file truncation point: the end of the last record we're keeping
        let data_truncate_len = if pos > 0 {
            // Find the highest data offset+length among records we're keeping
            let mut max_end: u64 = 0;
            for rec in &partition.records[..pos] {
                if let RecordDescriptor::Data { offset, length } = rec {
                    let end = offset + *length as u64;
                    if end > max_end {
                        max_end = end;
                    }
                }
            }
            max_end
        } else {
            0
        };

        partition.records.truncate(pos);
        partition.rebuild_filemark_index();

        // Remove redb record entries from the truncation point onward
        if let Err(e) = media_state.store.remove_records_from(partition_idx, pos as u64) {
            warn!(error = %e, "failed to remove truncated records from store");
        }

        // Truncate the data file
        if let Err(e) = media_state.store.truncate_data(data_truncate_len) {
            warn!(error = %e, "failed to truncate data file");
        }
    }
}
