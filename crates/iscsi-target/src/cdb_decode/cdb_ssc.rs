//! SSC (Sequential-Access / tape drive) specific CDB decoders.

use super::names::space_code_name;
use super::{control_byte_field, generic_cdb_fields, opcode_field, CdbField};
use super::cdb_common::common_cdb_fields;

pub fn decode_ssc_cdb(opcode: u8, cdb: &[u8]) -> Vec<CdbField> {
    if let Some(f) = common_cdb_fields(opcode, cdb) {
        return f;
    }
    match opcode {
        // REWIND
        0x01 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait for completion" }.into(),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // FORMAT MEDIUM
        0x04 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                let verify = (cdb[1] >> 1) & 0x01;
                let format = (cdb[2]) & 0x0F;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
                f.push(CdbField {
                    name: "VERIFY".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", verify),
                    decoded: if verify != 0 { "Verify after format" } else { "No verify" }.into(),
                });
                if cdb.len() >= 3 {
                    f.push(CdbField {
                        name: "Format".into(),
                        byte_offset: 2,
                        bit_range: Some("3:0".into()),
                        hex_value: format!("{:X}", format),
                        decoded: match format {
                            0x00 => "Use default format",
                            0x01 => "Partition medium",
                            0x02 => "Default format, then partition",
                            _ => "Reserved",
                        }.into(),
                    });
                }
            }
            if cdb.len() >= 5 {
                let xfer = u16::from_be_bytes([cdb[3], cdb[4]]);
                f.push(CdbField {
                    name: "Transfer Length".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:04X}", xfer),
                    decoded: format!("{} bytes", xfer),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // READ BLOCK LIMITS
        0x05 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // READ(6)
        0x08 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let sili = (cdb[1] >> 1) & 0x01;
                let fixed = cdb[1] & 0x01;
                let transfer_len =
                    ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "SILI".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", sili),
                    decoded: if sili != 0 { "Suppress ILI" } else { "Report ILI" }.into(),
                });
                f.push(CdbField {
                    name: "Fixed".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", u8::from(fixed != 0)),
                    decoded: if fixed != 0 { "Fixed-length blocks" } else { "Variable-length" }.into(),
                });
                f.push(CdbField {
                    name: "Transfer Length".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", transfer_len),
                    decoded: if fixed != 0 {
                        format!("{} blocks", transfer_len)
                    } else {
                        format!("{} bytes", transfer_len)
                    },
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // WRITE(6)
        0x0A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let fixed = cdb[1] & 0x01;
                let transfer_len =
                    ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "Fixed".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", u8::from(fixed != 0)),
                    decoded: if fixed != 0 { "Fixed-length blocks" } else { "Variable-length" }.into(),
                });
                f.push(CdbField {
                    name: "Transfer Length".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", transfer_len),
                    decoded: if fixed != 0 {
                        format!("{} blocks", transfer_len)
                    } else {
                        format!("{} bytes", transfer_len)
                    },
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // SET CAPACITY
        0x0B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 5 {
                let proportion = u16::from_be_bytes([cdb[3], cdb[4]]);
                f.push(CdbField {
                    name: "Proportion Value".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:04X}", proportion),
                    decoded: if proportion == 0 {
                        "Maximum native capacity".into()
                    } else {
                        format!("{} (proportion of capacity)", proportion)
                    },
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // WRITE FILEMARKS(6)
        0x10 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = (cdb[1] >> 0) & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 5 {
                let count = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "Filemark Count".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", count),
                    decoded: format!("{}", count),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // SPACE(6)
        0x11 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let code = cdb[1] & 0x07;
                let count_raw = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                let count = if count_raw & 0x800000 != 0 {
                    count_raw | 0xFF000000
                } else {
                    count_raw
                } as i32;
                f.push(CdbField {
                    name: "Code".into(),
                    byte_offset: 1,
                    bit_range: Some("2:0".into()),
                    hex_value: format!("{:X}", code),
                    decoded: space_code_name(code).into(),
                });
                f.push(CdbField {
                    name: "Count".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", count_raw),
                    decoded: format!("{}", count),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // VERIFY(6)
        0x13 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                let fixed = (cdb[1] >> 0) & 0x01;
                let vbf = (cdb[1] >> 2) & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
                f.push(CdbField {
                    name: "FIXED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", fixed),
                    decoded: if fixed != 0 { "Fixed block" } else { "Variable" }.into(),
                });
                f.push(CdbField {
                    name: "VBF".into(),
                    byte_offset: 1,
                    bit_range: Some("2".into()),
                    hex_value: format!("{}", vbf),
                    decoded: if vbf != 0 { "Verify by filemarks" } else { "By blocks" }.into(),
                });
            }
            if cdb.len() >= 5 {
                let len = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "Verification Length".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", len),
                    decoded: format!("{}", len),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // ERASE(6)
        0x19 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                let long = (cdb[1] >> 1) & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
                f.push(CdbField {
                    name: "LONG".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", long),
                    decoded: if long != 0 { "Erase to EOT" } else { "Erase single block" }.into(),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // LOAD UNLOAD
        0x1B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 5 {
                let hold = (cdb[4] >> 3) & 0x01;
                let eot = (cdb[4] >> 2) & 0x01;
                let reten = (cdb[4] >> 1) & 0x01;
                let load = cdb[4] & 0x01;
                f.push(CdbField {
                    name: "HOLD".into(),
                    byte_offset: 4,
                    bit_range: Some("3".into()),
                    hex_value: format!("{}", hold),
                    decoded: if hold != 0 { "Hold medium" } else { "Normal" }.into(),
                });
                f.push(CdbField {
                    name: "EOT".into(),
                    byte_offset: 4,
                    bit_range: Some("2".into()),
                    hex_value: format!("{}", eot),
                    decoded: if eot != 0 { "End of tape" } else { "Normal" }.into(),
                });
                f.push(CdbField {
                    name: "RETEN".into(),
                    byte_offset: 4,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", reten),
                    decoded: if reten != 0 { "Retension" } else { "No retension" }.into(),
                });
                f.push(CdbField {
                    name: "LOAD".into(),
                    byte_offset: 4,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", load),
                    decoded: if load != 0 { "Load" } else { "Unload" }.into(),
                });
            }
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // LOCATE(10)
        0x2B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let bt = (cdb[1] >> 2) & 0x01;
                let cp = (cdb[1] >> 1) & 0x01;
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "BT".into(),
                    byte_offset: 1,
                    bit_range: Some("2".into()),
                    hex_value: format!("{}", bt),
                    decoded: if bt != 0 { "Block address type" } else { "Logical object ID" }.into(),
                });
                f.push(CdbField {
                    name: "CP".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", cp),
                    decoded: if cp != 0 { "Change partition" } else { "Same partition" }.into(),
                });
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 8 {
                let block = u32::from_be_bytes([cdb[3], cdb[4], cdb[5], cdb[6]]);
                f.push(CdbField {
                    name: "Block Address".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:08X}", block),
                    decoded: format!("block {}", block),
                });
            }
            if cdb.len() >= 9 {
                f.push(CdbField {
                    name: "Partition".into(),
                    byte_offset: 8,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[8]),
                    decoded: format!("partition {}", cdb[8]),
                });
            }
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        // READ POSITION
        0x34 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let service_action = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", service_action),
                    decoded: match service_action {
                        0x00 => "Short form - block ID",
                        0x01 => "Long form",
                        0x06 => "Extended form",
                        0x08 => "Short form - vendor specific",
                        _ => "Unknown",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 10 {
                let alloc = u16::from_be_bytes([cdb[7], cdb[8]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:04X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
                f.push(control_byte_field(cdb, 9));
            }
            f
        }
        // REPORT DENSITY SUPPORT
        0x44 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let media = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "MEDIA".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", media),
                    decoded: if media != 0 { "Report media densities" } else { "Report drive densities" }.into(),
                });
            }
            if cdb.len() >= 9 {
                let alloc = u16::from_be_bytes([cdb[7], cdb[8]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:04X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        // WRITE FILEMARKS(16)
        0x80 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 14 {
                let count = u64::from_be_bytes([
                    cdb[4], cdb[5], cdb[6], cdb[7], cdb[8], cdb[9], cdb[10], cdb[11],
                ]);
                f.push(CdbField {
                    name: "Filemark Count".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:016X}", count),
                    decoded: format!("{}", count),
                });
            }
            if cdb.len() >= 16 { f.push(control_byte_field(cdb, 15)); }
            f
        }
        // ALLOW OVERWRITE
        0x82 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 3 {
                let allow = cdb[2] & 0x0F;
                f.push(CdbField {
                    name: "Allow Overwrite".into(),
                    byte_offset: 2,
                    bit_range: Some("3:0".into()),
                    hex_value: format!("{:X}", allow),
                    decoded: match allow {
                        0 => "Disabled",
                        1 => "Current position",
                        2 => "Format (CDB only)",
                        _ => "Reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 4 {
                f.push(CdbField {
                    name: "Partition".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[3]),
                    decoded: format!("partition {}", cdb[3]),
                });
            }
            if cdb.len() >= 12 {
                let loid = u64::from_be_bytes([
                    cdb[4], cdb[5], cdb[6], cdb[7], cdb[8], cdb[9], cdb[10], cdb[11],
                ]);
                f.push(CdbField {
                    name: "Logical Object Identifier".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:016X}", loid),
                    decoded: format!("{}", loid),
                });
            }
            if cdb.len() >= 16 { f.push(control_byte_field(cdb, 15)); }
            f
        }
        // READ ATTRIBUTE
        0x8C => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", sa),
                    decoded: match sa {
                        0x00 => "Attribute values",
                        0x01 => "Attribute list",
                        0x02 => "Volume list",
                        0x03 => "Partition list",
                        _ => "Reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 12 {
                let alloc = u32::from_be_bytes([cdb[10], cdb[11], cdb[12], cdb[13]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 10,
                    bit_range: None,
                    hex_value: format!("{:08X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 16 { f.push(control_byte_field(cdb, 15)); }
            f
        }
        // WRITE ATTRIBUTE
        0x8D => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let wtc = (cdb[1] >> 0) & 0x01;
                f.push(CdbField {
                    name: "WTC".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", wtc),
                    decoded: if wtc != 0 { "Write through cache" } else { "Normal" }.into(),
                });
            }
            if cdb.len() >= 16 {
                let len = u32::from_be_bytes([cdb[10], cdb[11], cdb[12], cdb[13]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 10,
                    bit_range: None,
                    hex_value: format!("{:08X}", len),
                    decoded: format!("{} bytes", len),
                });
                f.push(control_byte_field(cdb, 15));
            }
            f
        }
        // SPACE(16)
        0x91 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let code = cdb[1] & 0x07;
                f.push(CdbField {
                    name: "Code".into(),
                    byte_offset: 1,
                    bit_range: Some("2:0".into()),
                    hex_value: format!("{:X}", code),
                    decoded: space_code_name(code).into(),
                });
            }
            if cdb.len() >= 14 {
                let count = u64::from_be_bytes([
                    cdb[4], cdb[5], cdb[6], cdb[7], cdb[8], cdb[9], cdb[10], cdb[11],
                ]);
                f.push(CdbField {
                    name: "Count".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:016X}", count),
                    decoded: format!("{}", count as i64),
                });
            }
            if cdb.len() >= 14 {
                let param_len = u16::from_be_bytes([cdb[12], cdb[13]]);
                f.push(CdbField {
                    name: "Parameter Length".into(),
                    byte_offset: 12,
                    bit_range: None,
                    hex_value: format!("{:04X}", param_len),
                    decoded: format!("{} bytes", param_len),
                });
            }
            if cdb.len() >= 16 { f.push(control_byte_field(cdb, 15)); }
            f
        }
        // LOCATE(16)
        0x92 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let dest_type = (cdb[1] >> 2) & 0x07;
                let cp = (cdb[1] >> 1) & 0x01;
                let immed = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "DEST_TYPE".into(),
                    byte_offset: 1,
                    bit_range: Some("4:2".into()),
                    hex_value: format!("{}", dest_type),
                    decoded: match dest_type {
                        0 => "Logical object identifier",
                        1 => "Logical file identifier",
                        2 => "Logical set identifier",
                        3 => "End-of-data",
                        _ => "Reserved",
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "CP".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", cp),
                    decoded: if cp != 0 { "Change partition" } else { "Same partition" }.into(),
                });
                f.push(CdbField {
                    name: "IMMED".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", immed),
                    decoded: if immed != 0 { "Immediate" } else { "Wait" }.into(),
                });
            }
            if cdb.len() >= 4 {
                f.push(CdbField {
                    name: "Partition".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[3]),
                    decoded: format!("partition {}", cdb[3]),
                });
            }
            if cdb.len() >= 12 {
                let loid = u64::from_be_bytes([
                    cdb[4], cdb[5], cdb[6], cdb[7], cdb[8], cdb[9], cdb[10], cdb[11],
                ]);
                f.push(CdbField {
                    name: "Logical Identifier".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:016X}", loid),
                    decoded: format!("{}", loid),
                });
            }
            if cdb.len() >= 16 { f.push(control_byte_field(cdb, 15)); }
            f
        }
        _ => generic_cdb_fields(cdb),
    }
}
