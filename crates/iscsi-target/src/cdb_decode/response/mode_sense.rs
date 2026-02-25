//! Mode Sense(6) and Mode Sense(10) response data decoders.

use crate::cdb_decode::names::*;
use crate::cdb_decode::{data_field, data_field_parent, hex_string, DataField};
use super::mode_pages;

/// Decode MODE SENSE(6) response.
pub fn decode_mode_sense_6(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }

    // Mode Parameter Header (6-byte form, 4 bytes)
    let mode_data_len = data[0];
    let medium_type = data[1];
    let dev_specific = data[2];
    let bd_length = data[3];

    fields.push(data_field("Mode Data Length", 0, None, format!("{:02X}", mode_data_len), format!("{} bytes", mode_data_len)));
    fields.push(data_field("Medium Type", 1, None, format!("{:02X}", medium_type), format!("0x{:02X}", medium_type)));

    // Device-specific parameter: for tape, bit 7 = WP, bit 4 = DPO/FUA support
    let wp = (dev_specific >> 7) & 1;
    let dpofua = (dev_specific >> 4) & 1;
    fields.push(data_field("WP", 2, Some("7"), format!("{}", wp), if wp != 0 { "Write protected" } else { "Not write protected" }));
    fields.push(data_field("DPOFUA", 2, Some("4"), format!("{}", dpofua), if dpofua != 0 { "DPO/FUA supported" } else { "Not supported" }));
    fields.push(data_field("Block Descriptor Length", 3, None, format!("{:02X}", bd_length), format!("{} bytes", bd_length)));

    let mut offset = 4;

    // Block descriptor(s) — 8 bytes each for short form
    if bd_length > 0 && data.len() >= offset + bd_length as usize {
        let bd_end = offset + bd_length as usize;
        let mut bd_num = 1;
        while offset + 8 <= bd_end && offset + 8 <= data.len() {
            let density = data[offset];
            let num_blocks = ((data[offset + 1] as u32) << 16) | ((data[offset + 2] as u32) << 8) | (data[offset + 3] as u32);
            let block_len = ((data[offset + 5] as u32) << 16) | ((data[offset + 6] as u32) << 8) | (data[offset + 7] as u32);

            let children = vec![
                data_field("Density Code", offset, None, format!("{:02X}", density), density_code_name(density)),
                data_field("Number of Blocks", offset + 1, None, format!("{:06X}", num_blocks), if num_blocks == 0 { "All remaining".into() } else { format!("{}", num_blocks) }),
                data_field("Block Length", offset + 5, None, format!("{:06X}", block_len), if block_len == 0 { "Variable".into() } else { format!("{} bytes", block_len) }),
            ];
            fields.push(data_field_parent(format!("Block Descriptor {}", bd_num), offset, None, hex_string(&data[offset..offset + 8]), density_code_name(density), children));
            offset += 8;
            bd_num += 1;
        }
        offset = bd_end;
    }

    // Mode pages follow
    decode_mode_pages(data, offset, &mut fields);
    fields
}

/// Decode MODE SENSE(10) response.
pub fn decode_mode_sense_10(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 8 { return fields; }

    // Mode Parameter Header (10-byte form, 8 bytes)
    let mode_data_len = u16::from_be_bytes([data[0], data[1]]);
    let medium_type = data[2];
    let dev_specific = data[3];
    let longlba = (data[4] >> 0) & 0x01;
    let bd_length = u16::from_be_bytes([data[6], data[7]]);

    fields.push(data_field("Mode Data Length", 0, None, format!("{:04X}", mode_data_len), format!("{} bytes", mode_data_len)));
    fields.push(data_field("Medium Type", 2, None, format!("{:02X}", medium_type), format!("0x{:02X}", medium_type)));

    let wp = (dev_specific >> 7) & 1;
    let dpofua = (dev_specific >> 4) & 1;
    fields.push(data_field("WP", 3, Some("7"), format!("{}", wp), if wp != 0 { "Write protected" } else { "Not write protected" }));
    fields.push(data_field("DPOFUA", 3, Some("4"), format!("{}", dpofua), if dpofua != 0 { "DPO/FUA supported" } else { "Not supported" }));
    fields.push(data_field("LONGLBA", 4, Some("0"), format!("{}", longlba), if longlba != 0 { "Long LBA block descriptors" } else { "Short block descriptors" }));
    fields.push(data_field("Block Descriptor Length", 6, None, format!("{:04X}", bd_length), format!("{} bytes", bd_length)));

    let mut offset = 8;

    // Block descriptors
    if bd_length > 0 && data.len() >= offset + bd_length as usize {
        let bd_end = offset + bd_length as usize;
        let desc_size = if longlba != 0 { 16 } else { 8 };
        let mut bd_num = 1;
        while offset + desc_size <= bd_end && offset + desc_size <= data.len() {
            if desc_size == 8 {
                let density = data[offset];
                let num_blocks = ((data[offset + 1] as u32) << 16) | ((data[offset + 2] as u32) << 8) | (data[offset + 3] as u32);
                let block_len = ((data[offset + 5] as u32) << 16) | ((data[offset + 6] as u32) << 8) | (data[offset + 7] as u32);
                let children = vec![
                    data_field("Density Code", offset, None, format!("{:02X}", density), density_code_name(density)),
                    data_field("Number of Blocks", offset + 1, None, format!("{:06X}", num_blocks), if num_blocks == 0 { "All remaining".into() } else { format!("{}", num_blocks) }),
                    data_field("Block Length", offset + 5, None, format!("{:06X}", block_len), if block_len == 0 { "Variable".into() } else { format!("{} bytes", block_len) }),
                ];
                fields.push(data_field_parent(format!("Block Descriptor {}", bd_num), offset, None, hex_string(&data[offset..offset + 8]), density_code_name(density), children));
            } else {
                fields.push(data_field(format!("Long Block Descriptor {}", bd_num), offset, None, hex_string(&data[offset..offset + 16]), format!("{} bytes", desc_size)));
            }
            offset += desc_size;
            bd_num += 1;
        }
        offset = bd_end;
    }

    decode_mode_pages(data, offset, &mut fields);
    fields
}

fn decode_mode_pages(data: &[u8], start: usize, fields: &mut Vec<DataField>) {
    let mut offset = start;
    while offset + 2 <= data.len() {
        let ps = (data[offset] >> 7) & 1;
        let spf = (data[offset] >> 6) & 1;
        let page_code = data[offset] & 0x3F;
        let page_len = if spf != 0 && offset + 4 <= data.len() {
            u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize
        } else {
            data[offset + 1] as usize
        };

        let header_len = if spf != 0 { 4 } else { 2 };
        let page_end = (offset + header_len + page_len).min(data.len());
        let page_data = &data[offset..page_end];

        let page_name = mode_page_name(page_code);

        let mut children = vec![
            data_field("PS", offset, Some("7"), format!("{}", ps), if ps != 0 { "Saveable" } else { "Not saveable" }),
            data_field("SPF", offset, Some("6"), format!("{}", spf), if spf != 0 { "Sub-page format" } else { "Page_0 format" }),
            data_field("Page Code", offset, Some("5:0"), format!("{:02X}", page_code), page_name),
            data_field("Page Length", offset + 1, None, format!("{:02X}", page_len), format!("{} bytes", page_len)),
        ];

        // Decode page-specific fields
        let page_fields = mode_pages::decode_mode_page(page_code, page_data, offset);
        children.extend(page_fields);

        fields.push(data_field_parent(
            format!("Mode Page: {}", page_name),
            offset, None,
            hex_string(page_data),
            page_name,
            children,
        ));

        offset = page_end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_sense_6_header() {
        // Simple mode sense(6): 4-byte header, no block descriptors, one mode page
        let data = [
            0x0F, // mode data length = 15
            0x00, // medium type
            0x00, // device specific (not WP)
            0x00, // block descriptor length = 0
            // Data compression page (0x0F)
            0x0F, 0x0E, // page code, page length
            0xC0, // DCE=1, DCC=1
            0x80, // DDE=1
            0x00, 0x00, 0x00, 0x01, // compression algorithm
            0x00, 0x00, 0x00, 0x05, // decompression algorithm
            0x00, 0x00, 0x00, 0x00, // reserved
        ];
        let fields = decode_mode_sense_6(&data);
        assert!(fields.iter().any(|f| f.name == "Mode Data Length"));
        assert!(fields.iter().any(|f| f.name.contains("Data Compression")));
    }

    #[test]
    fn mode_sense_10_with_block_descriptor() {
        let mut data = vec![0u8; 28];
        // Header
        data[0] = 0x00; data[1] = 26; // mode data length
        data[2] = 0x00; // medium type
        data[3] = 0x00; // dev specific
        data[6] = 0x00; data[7] = 0x08; // BD length = 8
        // Block descriptor
        data[8] = 0x5E; // density LTO-8
        data[13] = 0x00; data[14] = 0x00; data[15] = 0x00; // block len = 0 (variable)
        // Mode page (control, 0x0A)
        data[16] = 0x0A; data[17] = 0x0A;

        let fields = decode_mode_sense_10(&data);
        assert!(fields.iter().any(|f| f.name.contains("Block Descriptor")));
    }
}
