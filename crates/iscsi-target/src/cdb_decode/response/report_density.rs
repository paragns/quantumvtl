//! REPORT DENSITY SUPPORT response decoder.

use crate::cdb_decode::names::density_code_name;
use crate::cdb_decode::{data_field, data_field_parent, hex_string, DataField};

pub fn decode(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }

    let avail_len = u16::from_be_bytes([data[0], data[1]]);
    fields.push(data_field("Available Density Support Length", 0, None, format!("{:04X}", avail_len), format!("{} bytes", avail_len)));

    // Density descriptors start at byte 4, each is 52 bytes
    let mut offset = 4;
    let mut desc_num = 1;
    while offset + 52 <= data.len() {
        let d = &data[offset..offset + 52];
        let primary = d[0];
        let secondary = d[1];
        let wrtok = (d[2] >> 7) & 1;
        let dup = (d[2] >> 6) & 1;
        let deflt = (d[2] >> 5) & 1;
        let dlv = d[2] & 0x01;
        let desc_len = u16::from_be_bytes([d[3], d[4]]);
        let bits_per_mm = ((d[5] as u32) << 16) | ((d[6] as u32) << 8) | (d[7] as u32);
        let media_width = u16::from_be_bytes([d[8], d[9]]);
        let tracks = u16::from_be_bytes([d[10], d[11]]);
        let capacity = u32::from_be_bytes([d[12], d[13], d[14], d[15]]);
        let org = ascii_field(&d[16..24]);
        let desc_name = ascii_field(&d[24..44]);
        let description = ascii_field(&d[44..52]);

        let children = vec![
            data_field("Primary Density Code", offset, None, format!("{:02X}", primary), density_code_name(primary)),
            data_field("Secondary Density Code", offset + 1, None, format!("{:02X}", secondary), density_code_name(secondary)),
            data_field("WRTOK", offset + 2, Some("7"), format!("{}", wrtok), if wrtok != 0 { "Write OK" } else { "Read only" }),
            data_field("DUP", offset + 2, Some("6"), format!("{}", dup), if dup != 0 { "Duplicate" } else { "Unique" }),
            data_field("DEFLT", offset + 2, Some("5"), format!("{}", deflt), if deflt != 0 { "Default density" } else { "Not default" }),
            data_field("DLV", offset + 2, Some("0"), format!("{}", dlv), if dlv != 0 { "DLV valid" } else { "DLV not valid" }),
            data_field("Descriptor Length", offset + 3, None, format!("{:04X}", desc_len), format!("{} bytes", desc_len)),
            data_field("Bits per mm", offset + 5, None, format!("{:06X}", bits_per_mm), format!("{}", bits_per_mm)),
            data_field("Media Width", offset + 8, None, format!("{:04X}", media_width), format!("{} tenths of mm", media_width)),
            data_field("Tracks", offset + 10, None, format!("{:04X}", tracks), format!("{}", tracks)),
            data_field("Capacity", offset + 12, None, format!("{:08X}", capacity), format!("{} MB", capacity)),
            data_field("Assigning Organization", offset + 16, None, hex_string(&d[16..24]), org),
            data_field("Density Name", offset + 24, None, hex_string(&d[24..44]), desc_name),
            data_field("Description", offset + 44, None, hex_string(&d[44..52]), description),
        ];

        let label = format!("{} (0x{:02X})", density_code_name(primary), primary);
        fields.push(data_field_parent(
            format!("Density Descriptor {}", desc_num),
            offset, None,
            hex_string(&data[offset..offset + 52]),
            label,
            children,
        ));

        offset += 52;
        desc_num += 1;
    }

    fields
}

fn ascii_field(data: &[u8]) -> String {
    let s: String = data.iter().map(|&b| if (0x20..=0x7E).contains(&b) { b as char } else { '.' }).collect();
    s.trim().to_string()
}
