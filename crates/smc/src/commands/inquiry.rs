//! INQUIRY command handler with VPD page dispatch.

use crate::sense::{self, SenseBuilder};
use crate::vpd;
use iscsi_target::ScsiResult;

/// Handle INQUIRY (12h).
pub fn handle_inquiry(
    cdb: &[u8],
    standard_inq: &[u8],
    serial: &str,
    vendor: &str,
    product: &str,
) -> ScsiResult {
    let evpd = cdb[1] & 0x01 != 0;
    let page_code = cdb[2];
    let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

    if !evpd {
        if page_code != 0x00 {
            return SenseBuilder::invalid_field_in_cdb().to_check_condition();
        }
        let mut data = standard_inq.to_vec();
        data.truncate(alloc_len);
        return sense::good_with_data(data);
    }

    // VPD page dispatch
    match vpd::handle_vpd_page(page_code, serial, vendor, product) {
        Some(mut data) => {
            data.truncate(alloc_len);
            sense::good_with_data(data)
        }
        None => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}
