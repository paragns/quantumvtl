//! READ(6) command handler.

use crate::media::tape::{DriveMediaState, TapeRecord};
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;

/// Handle READ(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length.
pub fn handle_read_6(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let _sili = cdb[1] & 0x02 != 0;
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    if transfer_length == 0 {
        return sense::good();
    }

    let pos = media_state.position.block_number as usize;
    let record_count = media_state.current_partition().records.len();

    // Check for EOD (blank check)
    if pos >= record_count {
        return SenseBuilder::end_of_data().to_check_condition();
    }

    // Check for filemark at current position
    if media_state.current_partition().records[pos].is_filemark() {
        media_state.position.block_number += 1;
        media_state.position.file_number += 1;
        return SenseBuilder::filemark_detected().to_check_condition();
    }

    if fixed {
        handle_read_fixed(transfer_length, media_state)
    } else {
        handle_read_variable(transfer_length, media_state)
    }
}

fn handle_read_fixed(block_count: u32, ms: &mut DriveMediaState) -> ScsiResult {
    // Get the block size from the first block we'll read
    let first_pos = ms.position.block_number as usize;
    let block_size = match &ms.current_partition().records[first_pos] {
        TapeRecord::Data(d) => d.len(),
        TapeRecord::Filemark => unreachable!(),
    };

    let mut data = Vec::with_capacity(block_size * block_count as usize);
    let mut blocks_read: u32 = 0;

    for _ in 0..block_count {
        let pos = ms.position.block_number as usize;
        let record_count = ms.current_partition().records.len();

        if pos >= record_count {
            break;
        }

        let is_filemark = ms.current_partition().records[pos].is_filemark();
        if is_filemark {
            break;
        }

        // Clone the data out, then update counters
        let block_data = match &ms.current_partition().records[pos] {
            TapeRecord::Data(d) => d.clone(),
            TapeRecord::Filemark => unreachable!(),
        };

        let byte_len = block_data.len() as u64;
        data.extend_from_slice(&block_data);
        ms.current_partition_mut().bytes_read_native += byte_len;
        ms.position.block_number += 1;
        blocks_read += 1;
    }

    if blocks_read < block_count {
        let residual = block_count - blocks_read;
        return SenseBuilder::filemark_detected()
            .with_information(residual)
            .to_check_condition_with_data(data);
    }

    sense::good_with_data(data)
}

fn handle_read_variable(max_bytes: u32, ms: &mut DriveMediaState) -> ScsiResult {
    let pos = ms.position.block_number as usize;

    let block_data = match &ms.current_partition().records[pos] {
        TapeRecord::Data(d) => d.clone(),
        TapeRecord::Filemark => unreachable!(),
    };

    let byte_len = block_data.len() as u64;
    ms.current_partition_mut().bytes_read_native += byte_len;
    ms.position.block_number += 1;

    let mut result_data = block_data;
    if result_data.len() > max_bytes as usize {
        result_data.truncate(max_bytes as usize);
    }

    sense::good_with_data(result_data)
}
