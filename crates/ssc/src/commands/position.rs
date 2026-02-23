//! Positioning commands: SPACE(6), REWIND, READ POSITION, LOCATE, READ BLOCK LIMITS.

use crate::media::tape::DriveMediaState;
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::trace;

/// Handle REWIND (01h).
pub fn handle_rewind(media_state: &mut DriveMediaState) -> ScsiResult {
    media_state.position.block_number = 0;
    media_state.position.file_number = 0;
    media_state.position.partition = 0;
    trace!("REWIND: position=0");
    sense::good()
}

/// Handle SPACE(6) (11h) — CDB[1] bits 0-2=code, CDB[2-4]=signed 24-bit count.
pub fn handle_space_6(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let code = cdb[1] & 0x07;
    let raw_count =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
    let count: i32 = if raw_count & 0x800000 != 0 {
        (raw_count | 0xFF000000) as i32
    } else {
        raw_count as i32
    };

    match code {
        0 => space_blocks(count, media_state),
        1 => space_filemarks(count, media_state),
        3 => {
            // Space to end-of-data
            let eod = media_state.current_partition().records.len() as u64;
            media_state.position.block_number = eod;
            trace!(position = eod, "SPACE to EOD");
            sense::good()
        }
        _ => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}

fn space_blocks(count: i32, ms: &mut DriveMediaState) -> ScsiResult {
    if count >= 0 {
        for _ in 0..count {
            let pos = ms.position.block_number as usize;
            if pos >= ms.current_partition().records.len() {
                return SenseBuilder::end_of_data().to_check_condition();
            }
            if ms.current_partition().records[pos].is_filemark() {
                ms.position.block_number += 1;
                ms.position.file_number += 1;
                return SenseBuilder::filemark_detected().to_check_condition();
            }
            ms.position.block_number += 1;
        }
    } else {
        for _ in 0..(-count) {
            if ms.position.block_number == 0 {
                return SenseBuilder::beginning_of_partition().to_check_condition();
            }
            ms.position.block_number -= 1;
            if ms.current_partition().records[ms.position.block_number as usize].is_filemark() {
                return SenseBuilder::filemark_detected().to_check_condition();
            }
        }
    }

    trace!(
        count,
        position = ms.position.block_number,
        "SPACE BLOCKS complete"
    );
    sense::good()
}

fn space_filemarks(count: i32, ms: &mut DriveMediaState) -> ScsiResult {
    if count >= 0 {
        let mut fm_seen = 0i32;
        while fm_seen < count {
            let pos = ms.position.block_number as usize;
            if pos >= ms.current_partition().records.len() {
                return SenseBuilder::end_of_data().to_check_condition();
            }
            if ms.current_partition().records[pos].is_filemark() {
                fm_seen += 1;
                ms.position.file_number += 1;
            }
            ms.position.block_number += 1;
        }
    } else {
        let mut fm_seen = 0i32;
        while fm_seen < -count {
            if ms.position.block_number == 0 {
                return SenseBuilder::beginning_of_partition().to_check_condition();
            }
            ms.position.block_number -= 1;
            if ms.current_partition().records[ms.position.block_number as usize].is_filemark() {
                fm_seen += 1;
                ms.position.file_number = ms.position.file_number.saturating_sub(1);
            }
        }
    }

    trace!(
        count,
        position = ms.position.block_number,
        "SPACE FILEMARKS complete"
    );
    sense::good()
}

/// Handle READ POSITION (34h).
pub fn handle_read_position(cdb: &[u8], media_state: &DriveMediaState) -> ScsiResult {
    let _service_action = cdb[1] & 0x1F;

    // Short form (service action 00h) — 20 bytes
    let pos = media_state.position.block_number as u32;
    let mut data = vec![0u8; 20];

    // Byte 0: BOP (bit 7), EOP (bit 6)
    if media_state.at_bop() {
        data[0] |= 0x80; // BOP
    }
    if media_state.at_eod() {
        data[0] |= 0x40; // EOP
    }

    // Bytes 4-7: first block location (BE32)
    data[4] = ((pos >> 24) & 0xFF) as u8;
    data[5] = ((pos >> 16) & 0xFF) as u8;
    data[6] = ((pos >> 8) & 0xFF) as u8;
    data[7] = (pos & 0xFF) as u8;

    // Bytes 8-11: last block location (same for now)
    data[8] = data[4];
    data[9] = data[5];
    data[10] = data[6];
    data[11] = data[7];

    sense::good_with_data(data)
}

/// Handle READ BLOCK LIMITS (05h) — 6 bytes.
pub fn handle_read_block_limits(media_state: &DriveMediaState) -> ScsiResult {
    let max_block = media_state.media.geometry.max_logical_block_bytes;
    let min_block = media_state.media.geometry.min_logical_block_bytes;

    let mut data = vec![0u8; 6];
    // Byte 0: granularity = 0
    // Bytes 1-3: maximum block length (BE24)
    data[1] = ((max_block >> 16) & 0xFF) as u8;
    data[2] = ((max_block >> 8) & 0xFF) as u8;
    data[3] = (max_block & 0xFF) as u8;
    // Bytes 4-5: minimum block length (BE16)
    data[4] = ((min_block >> 8) & 0xFF) as u8;
    data[5] = (min_block & 0xFF) as u8;

    sense::good_with_data(data)
}

/// Handle LOCATE(10) (2Bh) — stub.
pub fn handle_locate_10(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let _immed = cdb[1] & 0x01 != 0;
    let _cp = cdb[1] & 0x02 != 0;
    let dest_type = (cdb[1] >> 2) & 0x03;

    let block_address = ((cdb[3] as u64) << 24)
        | ((cdb[4] as u64) << 16)
        | ((cdb[5] as u64) << 8)
        | (cdb[6] as u64);

    match dest_type {
        0 => {
            // Position to logical object
            let total = media_state.current_partition().records.len() as u64;
            if block_address > total {
                // Seeking past EOD — position at EOD
                media_state.position.block_number = total;
            } else {
                media_state.position.block_number = block_address;
            }
        }
        1 => {
            // Position to filemark — find Nth filemark
            // TODO: proper filemark search
            media_state.position.block_number = block_address;
        }
        3 => {
            // Position to EOD
            media_state.position.block_number =
                media_state.current_partition().records.len() as u64;
        }
        _ => return SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }

    // Recompute file_number (approximate)
    media_state.position.file_number = count_filemarks_before(
        media_state.current_partition(),
        media_state.position.block_number,
    );

    trace!(
        block_address,
        position = media_state.position.block_number,
        "LOCATE complete"
    );
    sense::good()
}

/// Handle LOCATE(16) (92h) — stub.
pub fn handle_locate_16(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let _immed = cdb[1] & 0x01 != 0;
    let _cp = cdb[1] & 0x02 != 0;
    let dest_type = (cdb[1] >> 2) & 0x03;
    let _partition = cdb[3];

    let block_address = ((cdb[4] as u64) << 56)
        | ((cdb[5] as u64) << 48)
        | ((cdb[6] as u64) << 40)
        | ((cdb[7] as u64) << 32)
        | ((cdb[8] as u64) << 24)
        | ((cdb[9] as u64) << 16)
        | ((cdb[10] as u64) << 8)
        | (cdb[11] as u64);

    match dest_type {
        0 => {
            let total = media_state.current_partition().records.len() as u64;
            media_state.position.block_number = block_address.min(total);
        }
        3 => {
            media_state.position.block_number =
                media_state.current_partition().records.len() as u64;
        }
        _ => return SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }

    media_state.position.file_number = count_filemarks_before(
        media_state.current_partition(),
        media_state.position.block_number,
    );

    trace!(
        block_address,
        position = media_state.position.block_number,
        "LOCATE(16) complete"
    );
    sense::good()
}

/// Handle SPACE(16) (91h) — stub.
pub fn handle_space_16(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let code = cdb[1] & 0x07;

    // 8-byte count field (signed)
    let raw_count = i64::from_be_bytes([
        cdb[4], cdb[5], cdb[6], cdb[7], cdb[8], cdb[9], cdb[10], cdb[11],
    ]);

    // Delegate to the SPACE(6) logic with the larger count
    // For now, clamp to i32 range
    let count = raw_count.clamp(i32::MIN as i64, i32::MAX as i64) as i32;

    match code {
        0 => space_blocks(count, media_state),
        1 => space_filemarks(count, media_state),
        3 => {
            let eod = media_state.current_partition().records.len() as u64;
            media_state.position.block_number = eod;
            sense::good()
        }
        _ => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}

/// Count filemarks in partition before the given position.
fn count_filemarks_before(
    partition: &crate::media::tape::TapePartition,
    position: u64,
) -> u64 {
    partition
        .filemark_positions
        .iter()
        .filter(|&&fm| fm < position)
        .count() as u64
}
