//! MODE SENSE and MODE SELECT command handlers.

use crate::mode_pages::ModePageRegistry;
use crate::sense::{self, SenseBuilder};
use iscsi_target::ScsiResult;

/// Handle MODE SENSE(6) (1Ah).
pub fn handle_mode_sense_6(
    cdb: &[u8],
    registry: &ModePageRegistry,
) -> ScsiResult {
    let pc = cdb[2] >> 6;
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let alloc_len = cdb[4] as usize;

    let page_data = if page_code == 0x3F {
        registry.get_all_pages(pc)
    } else {
        match registry.get_page(page_code, subpage, pc) {
            Some(data) => data,
            None => return SenseBuilder::invalid_field_in_cdb().to_check_condition(),
        }
    };

    // MODE SENSE(6) header: 4 bytes, no block descriptors
    let mode_data_len = (3 + page_data.len()) as u8;
    let mut data = Vec::with_capacity(4 + page_data.len());
    data.push(mode_data_len); // Byte 0: Mode data length
    data.push(0x00);          // Byte 1: Medium type
    data.push(0x00);          // Byte 2: Device-specific parameter
    data.push(0x00);          // Byte 3: Block descriptor length
    data.extend_from_slice(&page_data);

    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle MODE SENSE(10) (5Ah).
pub fn handle_mode_sense_10(
    cdb: &[u8],
    registry: &ModePageRegistry,
) -> ScsiResult {
    let pc = cdb[2] >> 6;
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    let page_data = if page_code == 0x3F {
        registry.get_all_pages(pc)
    } else {
        match registry.get_page(page_code, subpage, pc) {
            Some(data) => data,
            None => return SenseBuilder::invalid_field_in_cdb().to_check_condition(),
        }
    };

    // MODE SENSE(10) header: 8 bytes, no block descriptors
    let mode_data_len = (6 + page_data.len()) as u16;
    let mut data = Vec::with_capacity(8 + page_data.len());
    data.push((mode_data_len >> 8) as u8); // Byte 0-1: Mode data length
    data.push(mode_data_len as u8);
    data.push(0x00); // Byte 2: Medium type
    data.push(0x00); // Byte 3: Device-specific parameter
    data.push(0x00); // Byte 4: Long LBA
    data.push(0x00); // Byte 5: Reserved
    data.push(0x00); // Byte 6-7: Block descriptor length
    data.push(0x00);
    data.extend_from_slice(&page_data);

    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle MODE SELECT(6) (15h).
///
/// The Quantum spec says the library has no changeable parameters.
/// We accept the command for compatibility but ignore the data.
pub fn handle_mode_select_6(cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
    let sp = cdb[1] & 0x01 != 0;

    if sp {
        return SenseBuilder::saving_parameters_not_supported().to_check_condition();
    }

    sense::good()
}

/// Handle MODE SELECT(10) (55h).
pub fn handle_mode_select_10(cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
    let sp = cdb[1] & 0x01 != 0;

    if sp {
        return SenseBuilder::saving_parameters_not_supported().to_check_condition();
    }

    sense::good()
}
