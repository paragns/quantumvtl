//! LOG SENSE (4Dh) command handler.

use crate::log_pages::LogPageRegistry;
use crate::sense::{self, SenseBuilder};
use iscsi_target::ScsiResult;

/// Handle LOG SENSE (4Dh).
pub fn handle_log_sense(cdb: &[u8], registry: &LogPageRegistry) -> ScsiResult {
    let _sp = cdb[1] & 0x01 != 0; // Save Parameters — must be 0
    let pc = (cdb[2] >> 6) & 0x03;
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let _parameter_pointer = ((cdb[5] as u16) << 8) | (cdb[6] as u16);
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    // Only cumulative values supported (PC=01b)
    if pc != 0x01 {
        return SenseBuilder::invalid_field_in_cdb().to_check_condition();
    }

    // Subpages not supported
    if subpage != 0x00 {
        return SenseBuilder::invalid_field_in_cdb().to_check_condition();
    }

    // Page 00h: Supported log pages
    if page_code == 0x00 {
        let supported = registry.supported_pages();
        let page_len = supported.len() as u16;
        let mut data = Vec::with_capacity(4 + supported.len());
        data.push(0x00); // Byte 0: Page code
        data.push(0x00); // Byte 1: Subpage code
        data.push((page_len >> 8) as u8); // Bytes 2-3: Page length
        data.push(page_len as u8);
        data.extend_from_slice(&supported);
        data.truncate(alloc_len);
        return sense::good_with_data(data);
    }

    // Get the requested page
    match registry.get_page(page_code) {
        Some(data) => {
            let mut data = data;
            data.truncate(alloc_len);
            sense::good_with_data(data)
        }
        None => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}
