//! READ ATTRIBUTE / WRITE ATTRIBUTE command handlers — stubs.

use crate::media::mam::MamAttributes;
use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;

/// Handle READ ATTRIBUTE (8Ch) — stub.
pub fn handle_read_attribute(cdb: &[u8], mam: &MamAttributes) -> ScsiResult {
    let service_action = cdb[1] & 0x1F;
    let alloc_len = ((cdb[10] as usize) << 24)
        | ((cdb[11] as usize) << 16)
        | ((cdb[12] as usize) << 8)
        | (cdb[13] as usize);

    match service_action {
        0x00 => {
            // Read attribute values
            let mut data = vec![0u8; 4]; // 4-byte header: available data length

            for attr in mam.all_attributes() {
                // Attribute header: id(2) + format(1) + length(2) + value(N)
                data.push(((attr.id >> 8) & 0xFF) as u8);
                data.push((attr.id & 0xFF) as u8);
                let format = if attr.read_only { 0x00 } else { 0x02 }; // binary or ASCII
                data.push(format);
                let len = attr.value.len() as u16;
                data.push(((len >> 8) & 0xFF) as u8);
                data.push((len & 0xFF) as u8);
                data.extend(&attr.value);
            }

            // Fix available data length
            let avail_len = (data.len() - 4) as u32;
            data[0] = ((avail_len >> 24) & 0xFF) as u8;
            data[1] = ((avail_len >> 16) & 0xFF) as u8;
            data[2] = ((avail_len >> 8) & 0xFF) as u8;
            data[3] = (avail_len & 0xFF) as u8;

            data.truncate(alloc_len);
            sense::good_with_data(data)
        }
        0x01 => {
            // Read attribute list (attribute IDs only)
            let mut data = vec![0u8; 4];
            for attr in mam.all_attributes() {
                data.push(((attr.id >> 8) & 0xFF) as u8);
                data.push((attr.id & 0xFF) as u8);
            }
            let avail_len = (data.len() - 4) as u32;
            data[0] = ((avail_len >> 24) & 0xFF) as u8;
            data[1] = ((avail_len >> 16) & 0xFF) as u8;
            data[2] = ((avail_len >> 8) & 0xFF) as u8;
            data[3] = (avail_len & 0xFF) as u8;
            data.truncate(alloc_len);
            sense::good_with_data(data)
        }
        _ => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}

/// Handle WRITE ATTRIBUTE (8Dh) — stub.
pub fn handle_write_attribute(cdb: &[u8], data_out: &[u8], mam: &mut MamAttributes) -> ScsiResult {
    let _param_list_len = ((cdb[10] as usize) << 24)
        | ((cdb[11] as usize) << 16)
        | ((cdb[12] as usize) << 8)
        | (cdb[13] as usize);

    // Parse attribute data from parameter list
    // Header: 4 bytes available data length, then attribute entries
    if data_out.len() < 4 {
        return SenseBuilder::invalid_field_in_parameter_list().to_check_condition();
    }

    let mut offset = 4; // skip header
    while offset + 5 <= data_out.len() {
        let attr_id = ((data_out[offset] as u16) << 8) | (data_out[offset + 1] as u16);
        let _format = data_out[offset + 2];
        let attr_len = ((data_out[offset + 3] as usize) << 8) | (data_out[offset + 4] as usize);
        offset += 5;

        if offset + attr_len > data_out.len() {
            break;
        }

        let value = data_out[offset..offset + attr_len].to_vec();
        if !mam.set(attr_id, value) {
            // Attribute is read-only
            return SenseBuilder::new(0x07, 0x27, 0x00).to_check_condition();
        }
        offset += attr_len;
    }

    sense::good()
}
