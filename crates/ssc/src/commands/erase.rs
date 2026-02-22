//! ERASE and FORMAT MEDIUM command handlers — stubs.

use crate::media::tape::DriveMediaState;
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::trace;

/// Handle ERASE(6) (19h) — stub.
pub fn handle_erase(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let long = cdb[1] & 0x01 != 0;

    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    if long {
        // Long erase: erase entire tape
        let partition = media_state.current_partition_mut();
        partition.records.clear();
        partition.filemark_positions.clear();
        media_state.position.block_number = 0;
        media_state.position.file_number = 0;
        trace!("ERASE (long): tape erased");
    } else {
        // Short erase: write EOD at current position
        let pos = media_state.position.block_number as usize;
        let partition = media_state.current_partition_mut();
        partition.records.truncate(pos);
        partition.rebuild_filemark_index();
        trace!(position = pos, "ERASE (short): EOD written at position");
    }

    sense::good()
}

/// Handle FORMAT MEDIUM (04h) — stub.
pub fn handle_format_medium(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let _immed = cdb[1] & 0x01 != 0;
    let _verify = cdb[1] & 0x02 != 0;
    let _format = cdb[2]; // Format type

    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    // For now, just clear the tape (single partition format)
    for partition in &mut media_state.media.partitions {
        partition.records.clear();
        partition.filemark_positions.clear();
        partition.bytes_written_native = 0;
        partition.bytes_written_compressed = 0;
        partition.bytes_read_native = 0;
    }
    media_state.position.block_number = 0;
    media_state.position.file_number = 0;
    media_state.position.partition = 0;

    // Mark media optimization as done (LTO-9)
    media_state.media.optimization_done = true;

    trace!("FORMAT MEDIUM: tape formatted");
    sense::good()
}
