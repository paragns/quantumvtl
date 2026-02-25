//! READ ELEMENT STATUS (SMC) response decoder with full nesting.

use crate::cdb_decode::names::element_type_name;
use crate::cdb_decode::{data_field, data_field_parent, hex_string, DataField};

pub fn decode(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 8 { return fields; }

    // Element Status Header (8 bytes)
    let first_addr = u16::from_be_bytes([data[0], data[1]]);
    let num_elements = u16::from_be_bytes([data[2], data[3]]);
    let report_len = ((data[5] as u32) << 16) | ((data[6] as u32) << 8) | (data[7] as u32);

    fields.push(data_field("First Element Address Reported", 0, None, format!("{:04X}", first_addr), format!("{}", first_addr)));
    fields.push(data_field("Number of Elements Available", 2, None, format!("{:04X}", num_elements), format!("{}", num_elements)));
    fields.push(data_field("Byte Count of Report", 5, None, format!("{:06X}", report_len), format!("{} bytes", report_len)));

    // Element Status Pages follow at offset 8
    let mut offset = 8;
    let mut page_num = 1;
    while offset + 8 <= data.len() {
        let elem_type = data[offset] & 0x0F;
        let pvoltag = (data[offset] >> 7) & 1;
        let avoltag = (data[offset] >> 6) & 1;
        let page_desc_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
        let page_byte_count = ((data[offset + 5] as u32) << 16) | ((data[offset + 6] as u32) << 8) | (data[offset + 7] as u32);

        let mut page_children = vec![
            data_field("Element Type", offset, Some("3:0"), format!("{:X}", elem_type), element_type_name(elem_type)),
            data_field("PVolTag", offset, Some("7"), format!("{}", pvoltag), if pvoltag != 0 { "Primary volume tag" } else { "No primary tag" }),
            data_field("AVolTag", offset, Some("6"), format!("{}", avoltag), if avoltag != 0 { "Alternate volume tag" } else { "No alternate tag" }),
            data_field("Element Descriptor Length", offset + 2, None, format!("{:04X}", page_desc_len), format!("{} bytes", page_desc_len)),
            data_field("Byte Count of Descriptors", offset + 5, None, format!("{:06X}", page_byte_count), format!("{} bytes", page_byte_count)),
        ];

        // Parse individual element descriptors
        let desc_start = offset + 8;
        let desc_end = (desc_start + page_byte_count as usize).min(data.len());
        let desc_len = page_desc_len as usize;

        if desc_len > 0 {
            let mut doff = desc_start;
            let mut elem_num = 1;
            while doff + desc_len <= desc_end {
                let elem_fields = decode_element_descriptor(&data[doff..doff + desc_len], doff, pvoltag != 0, avoltag != 0);
                let elem_addr = u16::from_be_bytes([data[doff], data[doff + 1]]);

                page_children.push(data_field_parent(
                    format!("Element {} (addr 0x{:04X})", elem_num, elem_addr),
                    doff, None,
                    hex_string(&data[doff..(doff + desc_len).min(data.len())]),
                    format!("{} element {}", element_type_name(elem_type), elem_addr),
                    elem_fields,
                ));

                doff += desc_len;
                elem_num += 1;
            }
        }

        fields.push(data_field_parent(
            format!("Element Status Page {} ({})", page_num, element_type_name(elem_type)),
            offset, None,
            format!("{} type={}", page_byte_count, elem_type),
            element_type_name(elem_type),
            page_children,
        ));

        offset = desc_end;
        page_num += 1;
    }

    fields
}

fn decode_element_descriptor(desc: &[u8], base: usize, pvoltag: bool, avoltag: bool) -> Vec<DataField> {
    let mut fields = Vec::new();
    if desc.len() < 4 { return fields; }

    let addr = u16::from_be_bytes([desc[0], desc[1]]);
    fields.push(data_field("Element Address", base, None, format!("{:04X}", addr), format!("{}", addr)));

    // Byte 2: flags
    let full = (desc[2] >> 0) & 1;
    let except = (desc[2] >> 2) & 1;
    let access = (desc[2] >> 3) & 1;
    let impexp = (desc[2] >> 1) & 1;
    fields.push(data_field("FULL", base + 2, Some("0"), format!("{}", full), if full != 0 { "Element full" } else { "Empty" }));
    fields.push(data_field("EXCEPT", base + 2, Some("2"), format!("{}", except), if except != 0 { "Abnormal state" } else { "Normal" }));
    fields.push(data_field("ACCESS", base + 2, Some("3"), format!("{}", access), if access != 0 { "Accessible" } else { "Not accessible" }));
    fields.push(data_field("IMPEXP", base + 2, Some("1"), format!("{}", impexp), if impexp != 0 { "Placed by operator" } else { "Placed by changer" }));

    // Byte 3: reserved
    // Byte 4-5: ASC/ASCQ (if EXCEPT=1)
    if desc.len() >= 6 {
        let asc = desc[4];
        let ascq = desc[5];
        if except != 0 || (asc != 0 || ascq != 0) {
            fields.push(data_field("ASC", base + 4, None, format!("{:02X}", asc), format!("0x{:02X}", asc)));
            fields.push(data_field("ASCQ", base + 5, None, format!("{:02X}", ascq), format!("0x{:02X}", ascq)));
        }
    }

    // Bytes 9-10: Source Element Address
    if desc.len() >= 11 {
        let svalid = (desc[9] >> 7) & 1;
        let invert = (desc[9] >> 6) & 1;
        let src_addr = u16::from_be_bytes([desc[10], if desc.len() > 11 { desc[11] } else { 0 }]);
        fields.push(data_field("SValid", base + 9, Some("7"), format!("{}", svalid), if svalid != 0 { "Source valid" } else { "Not valid" }));
        fields.push(data_field("Invert", base + 9, Some("6"), format!("{}", invert), if invert != 0 { "Inverted" } else { "Not inverted" }));
        if svalid != 0 {
            fields.push(data_field("Source Element Address", base + 10, None, format!("{:04X}", src_addr), format!("element {}", src_addr)));
        }
    }

    // Primary Volume Tag (36 bytes if pvoltag)
    let mut tag_offset = 12;
    if pvoltag && desc.len() >= tag_offset + 36 {
        let barcode = ascii_field(&desc[tag_offset..tag_offset + 32]);
        let vol_seq = u16::from_be_bytes([desc[tag_offset + 34], desc[tag_offset + 35]]);
        fields.push(data_field("Primary Volume Tag", base + tag_offset, None, hex_string(&desc[tag_offset..tag_offset + 32]), &barcode));
        fields.push(data_field("Volume Sequence Number", base + tag_offset + 34, None, format!("{:04X}", vol_seq), format!("{}", vol_seq)));
        tag_offset += 36;
    }

    // Alternate Volume Tag (36 bytes if avoltag)
    if avoltag && desc.len() >= tag_offset + 36 {
        let barcode = ascii_field(&desc[tag_offset..tag_offset + 32]);
        fields.push(data_field("Alternate Volume Tag", base + tag_offset, None, hex_string(&desc[tag_offset..tag_offset + 32]), barcode));
    }

    fields
}

fn ascii_field(data: &[u8]) -> String {
    let s: String = data.iter().map(|&b| if (0x20..=0x7E).contains(&b) { b as char } else { '.' }).collect();
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_status_header() {
        let mut data = vec![0u8; 80];
        // Header
        data[0] = 0x00; data[1] = 0x10; // first addr 16
        data[2] = 0x00; data[3] = 0x01; // 1 element
        data[5] = 0x00; data[6] = 0x00; data[7] = 72; // report length

        // Page header (8 bytes)
        data[8] = 0x02; // storage type
        data[10] = 0x00; data[11] = 64; // desc len = 64
        data[13] = 0x00; data[14] = 0x00; data[15] = 64; // byte count

        // Descriptor
        data[16] = 0x00; data[17] = 0x10; // address 16
        data[18] = 0x01; // FULL=1

        let fields = decode(&data);
        assert!(fields.iter().any(|f| f.name == "First Element Address Reported"));
        assert!(fields.iter().any(|f| f.name.contains("Element Status Page")));
    }
}
