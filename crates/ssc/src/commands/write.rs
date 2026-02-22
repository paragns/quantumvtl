//! WRITE(6) command handler.

use crate::media::tape::{DriveMediaState, TapeRecord};
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::trace;

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

    // Truncate from current position (overwrite mode)
    let partition = media_state.current_partition_mut();
    partition.records.truncate(pos);
    partition.rebuild_filemark_index();

    if fixed {
        // Fixed-block mode: data_out contains transfer_length blocks
        if transfer_length > 0 {
            let block_size = data_out.len() / transfer_length as usize;
            for i in 0..transfer_length as usize {
                let start = i * block_size;
                let end = start + block_size;
                let block = data_out[start..end].to_vec();
                let byte_len = block.len() as u64;
                partition.records.push(TapeRecord::Data(block));
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
        partition.records.push(TapeRecord::Data(data_out.to_vec()));
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
