//! WRITE FILEMARKS command handler.

use crate::media::tape::{DriveMediaState, RecordDescriptor};
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::{trace, warn};

/// Handle WRITE FILEMARKS(6) (10h) — CDB[2-4]=count.
pub fn handle_write_filemarks(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let count = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    // Count=0 means flush buffer without writing filemarks
    if count == 0 {
        // TODO: flush write buffer when buffering is implemented
        return sense::good();
    }

    let pos = media_state.position.block_number as usize;
    let partition_idx = media_state.position.partition as u32;

    // Truncate from current position
    {
        let partition = media_state.current_partition_mut();
        partition.records.truncate(pos);
    }
    // Remove truncated records from store
    if let Err(e) = media_state.store.remove_records_from(partition_idx, pos as u64) {
        warn!(error = %e, "failed to remove truncated records from store");
    }

    for _ in 0..count {
        let desc = RecordDescriptor::Filemark;
        let record_num = media_state.current_partition().records.len() as u64;

        if let Err(e) = media_state.store.save_record(partition_idx, record_num, &desc) {
            warn!(error = %e, "failed to save filemark record to store");
            return SenseBuilder::medium_error().to_check_condition();
        }

        let partition = media_state.current_partition_mut();
        let fm_pos = partition.records.len() as u64;
        partition.records.push(desc);
        partition.filemark_positions.push(fm_pos);
        media_state.position.block_number += 1;
        media_state.position.file_number += 1;
    }

    trace!(
        count,
        position = media_state.position.block_number,
        "WRITE FILEMARKS complete"
    );

    sense::good()
}
