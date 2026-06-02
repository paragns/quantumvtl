//! MODE SENSE and MODE SELECT command handlers.

use crate::mode_pages::{ModePageRegistry, PageControl};
use crate::sense;
use crate::ScsiResult;

/// Handle MODE SENSE(6) (1Ah).
pub fn handle_mode_sense_6(
    cdb: &[u8],
    registry: &ModePageRegistry,
    write_protected: bool,
) -> ScsiResult {
    let _dbd = cdb[1] & 0x08 != 0; // Disable Block Descriptors
    let pc = PageControl::from_byte(cdb[2]);
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let alloc_len = cdb[4] as usize;

    // Build block descriptor (8 bytes, variable block mode)
    let block_desc = vec![0u8; 8];
    // Density code in byte 0 — leave 0 for now
    // Bytes 1-3: number of blocks = 0 (variable)
    // Bytes 5-7: block length = 0 (variable block mode)

    // Get page data
    let page_data = if page_code == 0x3F {
        registry.get_all_pages(pc)
    } else {
        // Return empty page data for unsupported pages so initiators can still
        // read the write-protection bit from the header without failing.
        registry.get_page(page_code, subpage, pc).unwrap_or_default()
    };

    // Build mode parameter header (4 bytes for MODE SENSE(6))
    let total_len = 3 + block_desc.len() + page_data.len(); // excludes byte 0 itself
    let mut data = Vec::with_capacity(4 + block_desc.len() + page_data.len());

    // Byte 0: Mode data length
    data.push(total_len as u8);
    // Byte 1: Medium type = 0x00
    data.push(0x00);
    // Byte 2: Device-specific parameter
    //   bit 7 = WP (write-protected)
    //   bit 4 = BUFFERED MODE = 1 (buffered)
    let mut dsp = 0x10; // buffered mode = 1
    if write_protected {
        dsp |= 0x80;
    }
    data.push(dsp);
    // Byte 3: Block descriptor length
    data.push(block_desc.len() as u8);
    data.extend(&block_desc);
    data.extend(&page_data);

    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle MODE SENSE(10) (5Ah).
pub fn handle_mode_sense_10(
    cdb: &[u8],
    registry: &ModePageRegistry,
    write_protected: bool,
) -> ScsiResult {
    let _dbd = cdb[1] & 0x08 != 0;
    let pc = PageControl::from_byte(cdb[2]);
    let page_code = cdb[2] & 0x3F;
    let subpage = cdb[3];
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    let block_desc = vec![0u8; 8];

    let page_data = if page_code == 0x3F {
        registry.get_all_pages(pc)
    } else {
        registry.get_page(page_code, subpage, pc).unwrap_or_default()
    };

    // Build mode parameter header (8 bytes for MODE SENSE(10))
    let total_len = 6 + block_desc.len() + page_data.len() + 2; // +2 for length field itself
    let mode_data_len = total_len - 2; // everything after the 2-byte length field
    let mut data = Vec::with_capacity(total_len);

    // Bytes 0-1: Mode data length (BE16)
    data.push(((mode_data_len >> 8) & 0xFF) as u8);
    data.push((mode_data_len & 0xFF) as u8);
    // Byte 2: Medium type
    data.push(0x00);
    // Byte 3: Device-specific parameter
    let mut dsp = 0x10;
    if write_protected {
        dsp |= 0x80;
    }
    data.push(dsp);
    // Byte 4: LONGLBA
    data.push(0x00);
    // Byte 5: reserved
    data.push(0x00);
    // Bytes 6-7: Block descriptor length (BE16)
    data.push(0x00);
    data.push(block_desc.len() as u8);
    data.extend(&block_desc);
    data.extend(&page_data);

    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle MODE SELECT(6) (15h) — stub.
pub fn handle_mode_select_6(
    cdb: &[u8],
    data_out: &[u8],
    registry: &ModePageRegistry,
) -> ScsiResult {
    let _pf = cdb[1] & 0x10 != 0; // Page Format bit
    let _sp = cdb[1] & 0x01 != 0; // Save Pages bit
    let _param_list_len = cdb[4] as usize;

    // Skip mode parameter header (4 bytes) + block descriptors
    if data_out.len() < 4 {
        return SenseBuilder::invalid_field_in_parameter_list().to_check_condition();
    }
    let bd_len = data_out[3] as usize;
    let page_start = 4 + bd_len;
    if page_start >= data_out.len() {
        // No page data — OK
        return sense::good();
    }

    // Parse pages and apply
    let mut offset = page_start;
    while offset < data_out.len() {
        let page_code = data_out[offset] & 0x3F;
        let _spf = data_out[offset] & 0x40 != 0;
        if offset + 1 >= data_out.len() {
            break;
        }
        let page_len = data_out[offset + 1] as usize;
        let page_data_start = offset + 2;
        let page_data_end = page_data_start + page_len;
        if page_data_end > data_out.len() {
            break;
        }

        if let Err(msg) = registry.apply_select(page_code, 0, &data_out[page_data_start..page_data_end]) {
            tracing::warn!(page_code, error = msg, "MODE SELECT apply_select failed");
            return SenseBuilder::invalid_field_in_parameter_list().to_check_condition();
        }

        offset = page_data_end;
    }

    sense::good()
}

/// Handle MODE SELECT(10) (55h) — stub.
pub fn handle_mode_select_10(
    cdb: &[u8],
    data_out: &[u8],
    registry: &ModePageRegistry,
) -> ScsiResult {
    let _pf = cdb[1] & 0x10 != 0;
    let _sp = cdb[1] & 0x01 != 0;

    if data_out.len() < 8 {
        return SenseBuilder::invalid_field_in_parameter_list().to_check_condition();
    }
    let bd_len = ((data_out[6] as usize) << 8) | (data_out[7] as usize);
    let page_start = 8 + bd_len;
    if page_start >= data_out.len() {
        return sense::good();
    }

    let mut offset = page_start;
    while offset < data_out.len() {
        let page_code = data_out[offset] & 0x3F;
        if offset + 1 >= data_out.len() {
            break;
        }
        let page_len = data_out[offset + 1] as usize;
        let page_data_start = offset + 2;
        let page_data_end = page_data_start + page_len;
        if page_data_end > data_out.len() {
            break;
        }

        if let Err(msg) = registry.apply_select(page_code, 0, &data_out[page_data_start..page_data_end]) {
            tracing::warn!(page_code, error = msg, "MODE SELECT(10) apply_select failed");
            return SenseBuilder::invalid_field_in_parameter_list().to_check_condition();
        }
        offset = page_data_end;
    }

    sense::good()
}
