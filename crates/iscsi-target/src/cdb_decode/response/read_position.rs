//! READ POSITION response decoder: short, long, and extended forms.

use crate::cdb_decode::{data_field, hex_string, DataField};

/// Decode READ POSITION response based on service action.
pub fn decode(service_action: u8, data: &[u8]) -> Vec<DataField> {
    match service_action {
        0x00 | 0x08 => decode_short_form(data),
        0x01 => decode_long_form(data),
        0x06 => decode_extended_form(data),
        _ => {
            vec![data_field("Unknown Form", 0, None, hex_string(data), format!("SA=0x{:02X}, {} bytes", service_action, data.len()))]
        }
    }
}

fn decode_short_form(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 20 { return fields; }

    let bop = (data[0] >> 7) & 1;
    let eop = (data[0] >> 6) & 1;
    let locu = (data[0] >> 5) & 1;
    let bycu = (data[0] >> 4) & 1;
    let bpew = (data[0] >> 2) & 1;
    let perr = (data[0] >> 1) & 1;

    fields.push(data_field("BOP", 0, Some("7"), format!("{}", bop), if bop != 0 { "At beginning of partition" } else { "Not at BOP" }));
    fields.push(data_field("EOP", 0, Some("6"), format!("{}", eop), if eop != 0 { "At end of partition" } else { "Not at EOP" }));
    fields.push(data_field("LOCU", 0, Some("5"), format!("{}", locu), if locu != 0 { "Logical object count unknown" } else { "Count valid" }));
    fields.push(data_field("BYCU", 0, Some("4"), format!("{}", bycu), if bycu != 0 { "Byte count unknown" } else { "Byte count valid" }));
    fields.push(data_field("BPEW", 0, Some("2"), format!("{}", bpew), if bpew != 0 { "Beyond programmable early warning" } else { "Before early warning" }));
    fields.push(data_field("PERR", 0, Some("1"), format!("{}", perr), if perr != 0 { "Position error" } else { "No error" }));

    fields.push(data_field("Partition Number", 1, None, format!("{:02X}", data[1]), format!("{}", data[1])));

    let first_block = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let last_block = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let blocks_in_buffer = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) & 0x00FFFFFF;
    let num_bytes = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);

    fields.push(data_field("First Block Location", 4, None, format!("{:08X}", first_block), format!("block {}", first_block)));
    fields.push(data_field("Last Block Location", 8, None, format!("{:08X}", last_block), format!("block {}", last_block)));
    fields.push(data_field("Blocks in Buffer", 12, None, format!("{:06X}", blocks_in_buffer), format!("{}", blocks_in_buffer)));
    fields.push(data_field("Number of Bytes in Buffer", 16, None, format!("{:08X}", num_bytes), format!("{}", num_bytes)));

    fields
}

fn decode_long_form(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 32 { return fields; }

    let bop = (data[0] >> 7) & 1;
    let eop = (data[0] >> 6) & 1;
    let mpu = (data[0] >> 3) & 1;
    let bcu = (data[0] >> 2) & 1;
    let bpew = (data[0] >> 2) & 1;

    fields.push(data_field("BOP", 0, Some("7"), format!("{}", bop), if bop != 0 { "At BOP" } else { "Not at BOP" }));
    fields.push(data_field("EOP", 0, Some("6"), format!("{}", eop), if eop != 0 { "At EOP" } else { "Not at EOP" }));
    fields.push(data_field("MPU", 0, Some("3"), format!("{}", mpu), if mpu != 0 { "Mark position unknown" } else { "Known" }));
    fields.push(data_field("BCU", 0, Some("2"), format!("{}", bcu), if bcu != 0 { "Block count unknown" } else { "Known" }));
    fields.push(data_field("BPEW", 0, Some("2"), format!("{}", bpew), if bpew != 0 { "Beyond early warning" } else { "Before" }));

    fields.push(data_field("Partition Number", 1, None, format!("{:02X}", data[1]), format!("{}", data[1])));

    let block_num = u64::from_be_bytes([data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11]]);
    let file_num = u64::from_be_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]);
    let set_num = u64::from_be_bytes([data[20], data[21], data[22], data[23], data[24], data[25], data[26], data[27]]);

    fields.push(data_field("Block Number", 4, None, format!("{:016X}", block_num), format!("{}", block_num)));
    fields.push(data_field("File Number", 12, None, format!("{:016X}", file_num), format!("{}", file_num)));
    fields.push(data_field("Set Number", 20, None, format!("{:016X}", set_num), format!("{}", set_num)));

    fields
}

fn decode_extended_form(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 32 { return fields; }

    let bop = (data[0] >> 7) & 1;
    let eop = (data[0] >> 6) & 1;
    let locu = (data[0] >> 5) & 1;
    let bycu = (data[0] >> 4) & 1;
    let bpew = (data[0] >> 2) & 1;
    let perr = (data[0] >> 1) & 1;

    fields.push(data_field("BOP", 0, Some("7"), format!("{}", bop), if bop != 0 { "At BOP" } else { "Not at BOP" }));
    fields.push(data_field("EOP", 0, Some("6"), format!("{}", eop), if eop != 0 { "At EOP" } else { "Not at EOP" }));
    fields.push(data_field("LOCU", 0, Some("5"), format!("{}", locu), if locu != 0 { "Count unknown" } else { "Count valid" }));
    fields.push(data_field("BYCU", 0, Some("4"), format!("{}", bycu), if bycu != 0 { "Byte count unknown" } else { "Valid" }));
    fields.push(data_field("BPEW", 0, Some("2"), format!("{}", bpew), if bpew != 0 { "Beyond early warning" } else { "Before" }));
    fields.push(data_field("PERR", 0, Some("1"), format!("{}", perr), if perr != 0 { "Position error" } else { "No error" }));

    fields.push(data_field("Partition Number", 1, None, format!("{:02X}", data[1]), format!("{}", data[1])));

    let addl_len = u16::from_be_bytes([data[2], data[3]]);
    fields.push(data_field("Additional Length", 2, None, format!("{:04X}", addl_len), format!("{} bytes", addl_len)));

    let blocks_in_buffer = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) & 0x00FFFFFF;
    fields.push(data_field("Blocks in Buffer", 5, None, format!("{:06X}", blocks_in_buffer), format!("{}", blocks_in_buffer)));

    let first_loid = u64::from_be_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]);
    fields.push(data_field("First Logical Object Location", 8, None, format!("{:016X}", first_loid), format!("{}", first_loid)));

    let last_loid = u64::from_be_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]);
    fields.push(data_field("Last Logical Object Location", 16, None, format!("{:016X}", last_loid), format!("{}", last_loid)));

    if data.len() >= 32 {
        let bytes_in_buf = u64::from_be_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]);
        fields.push(data_field("Bytes in Buffer", 24, None, format!("{:016X}", bytes_in_buf), format!("{}", bytes_in_buf)));
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_form_position() {
        let mut data = [0u8; 20];
        data[0] = 0x80; // BOP=1
        data[1] = 0;    // partition 0
        // first block = 0
        // last block = 0
        let fields = decode(0x00, &data);
        assert!(fields.iter().any(|f| f.name == "BOP" && f.decoded.contains("beginning")));
        assert!(fields.iter().any(|f| f.name == "First Block Location"));
    }
}
