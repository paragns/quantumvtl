//! ERASE and FORMAT MEDIUM command handlers.

use crate::media::tape::DriveMediaState;
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::{trace, warn};

/// Handle ERASE(6) (19h).
pub fn handle_erase(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let long = cdb[1] & 0x01 != 0;

    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    let partition_idx = media_state.position.partition as u32;

    if long {
        // Long erase: erase entire partition
        let partition = media_state.current_partition_mut();
        partition.records.clear();
        partition.filemark_positions.clear();
        media_state.position.block_number = 0;
        media_state.position.file_number = 0;

        // Clear store
        if let Err(e) = media_state.store.clear_partition_records(partition_idx) {
            warn!(error = %e, "failed to clear partition records in store");
        }
        if let Err(e) = media_state.store.truncate_data(0) {
            warn!(error = %e, "failed to truncate data file");
        }

        trace!("ERASE (long): tape erased");
    } else {
        // Short erase: write EOD at current position
        let pos = media_state.position.block_number as usize;
        let partition = media_state.current_partition_mut();
        let old_len = partition.records.len();

        if pos < old_len {
            // Calculate data file truncation point
            use crate::media::tape::RecordDescriptor;
            let mut max_end: u64 = 0;
            for rec in &partition.records[..pos] {
                if let RecordDescriptor::Data { offset, length } = rec {
                    let end = offset + *length as u64;
                    if end > max_end {
                        max_end = end;
                    }
                }
            }

            partition.records.truncate(pos);
            partition.rebuild_filemark_index();

            if let Err(e) = media_state.store.remove_records_from(partition_idx, pos as u64) {
                warn!(error = %e, "failed to remove records from store");
            }
            if let Err(e) = media_state.store.truncate_data(max_end) {
                warn!(error = %e, "failed to truncate data file");
            }
        }

        trace!(position = pos, "ERASE (short): EOD written at position");
    }

    sense::good()
}

/// Handle FORMAT MEDIUM (04h).
pub fn handle_format_medium(cdb: &[u8], media_state: &mut DriveMediaState) -> ScsiResult {
    let _immed = cdb[1] & 0x01 != 0;
    let _verify = cdb[1] & 0x02 != 0;
    let _format = cdb[2]; // Format type

    if media_state.media.write_protected {
        return SenseBuilder::write_protected().to_check_condition();
    }

    // Clear all partitions
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

    // Clear store
    if let Err(e) = media_state.store.clear_all_records() {
        warn!(error = %e, "failed to clear all records in store");
    }
    if let Err(e) = media_state.store.truncate_data(0) {
        warn!(error = %e, "failed to truncate data file");
    }
    if let Err(e) = media_state.store.clear_all_partition_stats() {
        warn!(error = %e, "failed to clear partition stats in store");
    }

    // Mark media optimization as done (LTO-9)
    media_state.media.optimization_done = true;

    trace!("FORMAT MEDIUM: tape formatted");
    sense::good()
}
