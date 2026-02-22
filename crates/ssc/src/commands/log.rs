//! LOG SENSE and LOG SELECT command handlers.

use crate::log_pages::LogPageRegistry;
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;

/// Handle LOG SENSE (4Dh).
pub fn handle_log_sense(cdb: &[u8], registry: &LogPageRegistry) -> ScsiResult {
    let _ppc = cdb[1] & 0x02 != 0; // Parameter Pointer Control
    let _sp = cdb[1] & 0x01 != 0;  // Saving Parameters
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    let data = if page_code == 0x00 && subpage == 0x00 {
        // Supported log pages
        registry.supported_pages()
    } else {
        match registry.get_page(page_code, subpage) {
            Some(data) => data,
            None => return SenseBuilder::invalid_field_in_cdb().to_check_condition(),
        }
    };

    let mut result = data;
    result.truncate(alloc_len);
    sense::good_with_data(result)
}

/// Handle LOG SELECT (4Ch) — stub.
pub fn handle_log_select(cdb: &[u8], registry: &LogPageRegistry) -> ScsiResult {
    let _pcr = cdb[1] & 0x02 != 0; // Parameter Code Reset
    let _sp = cdb[1] & 0x01 != 0;
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];

    if page_code == 0x00 {
        registry.reset_all();
    } else {
        registry.reset_page(page_code, subpage);
    }

    sense::good()
}
