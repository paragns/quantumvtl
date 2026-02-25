//! READ BLOCK LIMITS response decoder.

use crate::cdb_decode::{data_field, DataField};

pub fn decode(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 6 { return fields; }

    let granularity = data[0] & 0x1F;
    fields.push(data_field("Granularity", 0, Some("4:0"), format!("{:02X}", granularity), format!("2^{} = {} bytes", granularity, 1u32 << granularity)));

    let max_block_len = ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
    fields.push(data_field("Maximum Block Length", 1, None, format!("{:06X}", max_block_len),
        if max_block_len == 0 { "No limit".into() } else { format!("{} bytes ({:.1} MB)", max_block_len, max_block_len as f64 / 1_048_576.0) }));

    let min_block_len = u16::from_be_bytes([data[4], data[5]]);
    fields.push(data_field("Minimum Block Length", 4, None, format!("{:04X}", min_block_len), format!("{} bytes", min_block_len)));

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_limits() {
        let data = [
            0x00,             // granularity = 0
            0x08, 0x00, 0x00, // max = 512KB
            0x00, 0x01,       // min = 1 byte
        ];
        let fields = decode(&data);
        assert!(fields.iter().any(|f| f.name == "Maximum Block Length"));
        assert!(fields.iter().any(|f| f.name == "Minimum Block Length" && f.decoded.contains("1")));
    }
}
