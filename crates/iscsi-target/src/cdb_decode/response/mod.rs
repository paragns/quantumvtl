//! Response data decoders — dispatches by opcode + CDB context to parse
//! Data-In payloads into structured `DataField` trees.

mod inquiry;
mod mode_sense;
mod mode_pages;
mod log_sense;
mod log_pages;
mod read_position;
mod read_block_limits;
mod report_density;
mod element_status;
mod report_luns;

use super::DataField;

/// Decode response Data-In bytes based on the opcode and full CDB.
/// Returns None if there is no decoder for this opcode or the data is empty.
pub fn decode_response_data(opcode: u8, cdb: &[u8], data: &[u8]) -> Option<Vec<DataField>> {
    if data.is_empty() {
        return None;
    }
    match opcode {
        // INQUIRY
        0x12 => {
            let evpd = cdb.get(1).map(|b| b & 0x01).unwrap_or(0);
            if evpd != 0 {
                let page = cdb.get(2).copied().unwrap_or(0);
                Some(inquiry::decode_vpd_page(page, data))
            } else {
                Some(inquiry::decode_standard_inquiry(data))
            }
        }
        // REQUEST SENSE
        0x03 => Some(super::sense::decode_sense(data).fields.unwrap_or_default()),
        // READ BLOCK LIMITS
        0x05 => Some(read_block_limits::decode(data)),
        // MODE SENSE(6)
        0x1A => Some(mode_sense::decode_mode_sense_6(data)),
        // READ POSITION
        0x34 => {
            let sa = cdb.get(1).map(|b| b & 0x1F).unwrap_or(0);
            Some(read_position::decode(sa, data))
        }
        // REPORT DENSITY SUPPORT
        0x44 => Some(report_density::decode(data)),
        // LOG SENSE
        0x4D => Some(log_sense::decode(data)),
        // MODE SENSE(10)
        0x5A => Some(mode_sense::decode_mode_sense_10(data)),
        // REPORT LUNS
        0xA0 => Some(report_luns::decode(data)),
        // READ ELEMENT STATUS (SMC)
        0xB8 => Some(element_status::decode(data)),
        _ => None,
    }
}
