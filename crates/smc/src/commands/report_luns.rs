//! REPORT LUNS (A0h) command handler.

use crate::sense;
use iscsi_target::ScsiResult;

/// Handle REPORT LUNS (A0h).
///
/// The media changer reports only itself (a single LUN).
pub fn handle_report_luns(cdb: &[u8]) -> ScsiResult {
    let alloc_len = ((cdb[6] as usize) << 24)
        | ((cdb[7] as usize) << 16)
        | ((cdb[8] as usize) << 8)
        | (cdb[9] as usize);

    // Response: 4-byte LUN list length + 4 reserved + one 8-byte LUN
    let mut data = vec![0u8; 16];
    // Bytes 0-3: LUN list length (8 bytes = 1 LUN)
    data[3] = 0x08;
    // Bytes 4-7: Reserved (already 0)
    // Bytes 8-15: LUN 0 (all zeros = LUN 0 in single-level addressing)

    data.truncate(alloc_len);
    sense::good_with_data(data)
}
