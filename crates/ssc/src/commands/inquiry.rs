//! INQUIRY command handler — standard and VPD dispatch.

use crate::sense::{self};
use crate::vpd;
use crate::ScsiResult;

/// Handle INQUIRY (12h).
pub fn handle_inquiry(cdb: &[u8], standard_inq: &[u8], serial: &str) -> ScsiResult {
    let evpd = cdb[1] & 0x01 != 0;
    let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

    if !evpd {
        // Standard inquiry
        let mut data = standard_inq.to_vec();
        data.truncate(alloc_len);
        return sense::good_with_data(data);
    }

    // VPD page
    let page_code = cdb[2];
    vpd::handle_vpd_page(page_code, alloc_len, serial)
}
