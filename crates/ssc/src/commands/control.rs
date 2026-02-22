//! Control commands: TEST UNIT READY, REQUEST SENSE, LOAD/UNLOAD,
//! PREVENT/ALLOW MEDIUM REMOVAL, ALLOW OVERWRITE.

use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;
use tracing::trace;

/// Handle TEST UNIT READY (00h). Called from lib.rs before we have media state.
/// Returns None if media is loaded (caller proceeds), or Some(result) if not ready.
pub fn handle_test_unit_ready(media_loaded: bool) -> ScsiResult {
    if !media_loaded {
        return SenseBuilder::no_media().to_check_condition();
    }
    sense::good()
}

/// Handle REQUEST SENSE (03h).
/// TODO: Return pending unit attention or deferred sense when available.
pub fn handle_request_sense(cdb: &[u8]) -> ScsiResult {
    let alloc_len = cdb[4] as usize;
    let mut data = SenseBuilder::no_sense().build();
    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle LOAD/UNLOAD (1Bh) — stub.
/// In the real implementation, the changer handles load/unload via MediaLoadNotify.
/// This command handles the case where the initiator sends it directly to the drive.
pub fn handle_load_unload(cdb: &[u8]) -> ScsiResult {
    let _immed = cdb[1] & 0x01 != 0;
    let _hold = cdb[4] & 0x08 != 0;
    let _reten = cdb[4] & 0x02 != 0;
    let load = cdb[4] & 0x01 != 0;

    trace!(load, "LOAD/UNLOAD");

    // For now, just return GOOD. The actual media loading is handled
    // by the changer's MediaLoadNotify callbacks.
    sense::good()
}

/// Handle PREVENT/ALLOW MEDIUM REMOVAL (1Eh) — stub.
pub fn handle_prevent_allow_medium_removal(cdb: &[u8]) -> ScsiResult {
    let _prevent = cdb[4] & 0x03;
    trace!("PREVENT/ALLOW MEDIUM REMOVAL");
    // TODO: Track prevent state for LOAD/UNLOAD enforcement
    sense::good()
}

/// Handle ALLOW OVERWRITE (82h) — stub.
pub fn handle_allow_overwrite(cdb: &[u8]) -> ScsiResult {
    let _allow = cdb[2] & 0x07;
    trace!("ALLOW OVERWRITE");
    // TODO: Implement append-only mode validation
    sense::good()
}
