//! REPORT LUNS response decoder.

use crate::cdb_decode::{data_field, hex_string, DataField};

pub fn decode(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 8 { return fields; }

    let lun_list_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    fields.push(data_field("LUN List Length", 0, None, format!("{:08X}", lun_list_len), format!("{} bytes ({} LUNs)", lun_list_len, lun_list_len / 8)));

    // Reserved bytes 4-7
    // LUN descriptors start at byte 8, each is 8 bytes
    let mut offset = 8;
    let mut lun_num = 0;
    let end = (8 + lun_list_len as usize).min(data.len());
    while offset + 8 <= end {
        let lun_bytes = &data[offset..offset + 8];
        let lun_value = u16::from_be_bytes([lun_bytes[0], lun_bytes[1]]);
        let addr_method = (lun_bytes[0] >> 6) & 0x03;

        let decoded = match addr_method {
            0x00 => format!("LUN {} (peripheral device addressing)", lun_value & 0x3FFF),
            0x01 => format!("LUN {} (flat space addressing)", lun_value & 0x3FFF),
            0x02 => format!("LUN (logical unit addressing)"),
            0x03 => format!("LUN (extended logical unit addressing)"),
            _ => format!("LUN 0x{:04X}", lun_value),
        };

        fields.push(data_field(
            format!("LUN {}", lun_num),
            offset, None,
            hex_string(lun_bytes),
            decoded,
        ));

        offset += 8;
        lun_num += 1;
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_luns_two_luns() {
        let mut data = vec![0u8; 24];
        // LUN list length = 16 (2 LUNs)
        data[0] = 0x00; data[1] = 0x00; data[2] = 0x00; data[3] = 0x10;
        // LUN 0
        data[8] = 0x00; data[9] = 0x00;
        // LUN 1
        data[16] = 0x00; data[17] = 0x01;

        let fields = decode(&data);
        assert!(fields.iter().any(|f| f.name == "LUN List Length" && f.decoded.contains("2 LUNs")));
        // "LUN List Length" + "LUN 0" + "LUN 1" — filter only LUN descriptors
        assert_eq!(fields.iter().filter(|f| f.name.starts_with("LUN ") && !f.name.contains("List")).count(), 2);
    }
}
