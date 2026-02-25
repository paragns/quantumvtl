//! SMC (Media Changer) specific CDB decoders.

use super::names::element_type_name;
use super::{control_byte_field, generic_cdb_fields, opcode_field, CdbField};
use super::cdb_common::common_cdb_fields;

pub fn decode_smc_cdb(opcode: u8, cdb: &[u8]) -> Vec<CdbField> {
    if let Some(f) = common_cdb_fields(opcode, cdb) {
        return f;
    }
    match opcode {
        // INITIALIZE ELEMENT STATUS
        0x07 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // RESERVE ELEMENT(6)
        0x16 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // RELEASE ELEMENT(6)
        0x17 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 6 { f.push(control_byte_field(cdb, 5)); }
            f
        }
        // POSITION TO ELEMENT
        0x2B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 4 {
                let transport = u16::from_be_bytes([cdb[2], cdb[3]]);
                f.push(CdbField {
                    name: "Transport Element Address".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", transport),
                    decoded: format!("element {}", transport),
                });
            }
            if cdb.len() >= 6 {
                let dest = u16::from_be_bytes([cdb[4], cdb[5]]);
                f.push(CdbField {
                    name: "Destination Element Address".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", dest),
                    decoded: format!("element {}", dest),
                });
            }
            if cdb.len() >= 2 {
                let invert = (cdb[1] >> 0) & 0x01;
                f.push(CdbField {
                    name: "Invert".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", invert),
                    decoded: if invert != 0 { "Invert medium" } else { "No invert" }.into(),
                });
            }
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        // RESERVE ELEMENT(10)
        0x56 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        // RELEASE ELEMENT(10)
        0x57 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        // MOVE MEDIUM
        0xA5 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 12 {
                let transport = u16::from_be_bytes([cdb[2], cdb[3]]);
                let source = u16::from_be_bytes([cdb[4], cdb[5]]);
                let dest = u16::from_be_bytes([cdb[6], cdb[7]]);
                let invert = (cdb[10] >> 0) & 0x01;
                f.push(CdbField {
                    name: "Transport Address".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", transport),
                    decoded: format!("element {}", transport),
                });
                f.push(CdbField {
                    name: "Source Address".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", source),
                    decoded: format!("element {}", source),
                });
                f.push(CdbField {
                    name: "Destination Address".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:04X}", dest),
                    decoded: format!("element {}", dest),
                });
                f.push(CdbField {
                    name: "Invert".into(),
                    byte_offset: 10,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", invert),
                    decoded: if invert != 0 { "Invert medium" } else { "No invert" }.into(),
                });
                f.push(control_byte_field(cdb, 11));
            }
            f
        }
        // EXCHANGE MEDIUM
        0xA6 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 12 {
                let transport = u16::from_be_bytes([cdb[2], cdb[3]]);
                let source = u16::from_be_bytes([cdb[4], cdb[5]]);
                let dest1 = u16::from_be_bytes([cdb[6], cdb[7]]);
                let dest2 = u16::from_be_bytes([cdb[8], cdb[9]]);
                let inv1 = (cdb[10] >> 1) & 0x01;
                let inv2 = cdb[10] & 0x01;
                f.push(CdbField {
                    name: "Transport Address".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", transport),
                    decoded: format!("element {}", transport),
                });
                f.push(CdbField {
                    name: "Source Address".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", source),
                    decoded: format!("element {}", source),
                });
                f.push(CdbField {
                    name: "First Destination".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:04X}", dest1),
                    decoded: format!("element {}", dest1),
                });
                f.push(CdbField {
                    name: "Second Destination".into(),
                    byte_offset: 8,
                    bit_range: None,
                    hex_value: format!("{:04X}", dest2),
                    decoded: format!("element {}", dest2),
                });
                f.push(CdbField {
                    name: "INV1".into(),
                    byte_offset: 10,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", inv1),
                    decoded: if inv1 != 0 { "Invert first" } else { "No invert" }.into(),
                });
                f.push(CdbField {
                    name: "INV2".into(),
                    byte_offset: 10,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", inv2),
                    decoded: if inv2 != 0 { "Invert second" } else { "No invert" }.into(),
                });
                f.push(control_byte_field(cdb, 11));
            }
            f
        }
        // REQUEST VOLUME ELEMENT ADDRESS
        0xB5 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let elem_type = cdb[1] & 0x0F;
                f.push(CdbField {
                    name: "Element Type".into(),
                    byte_offset: 1,
                    bit_range: Some("3:0".into()),
                    hex_value: format!("{:X}", elem_type),
                    decoded: element_type_name(elem_type).into(),
                });
            }
            if cdb.len() >= 6 {
                let start = u16::from_be_bytes([cdb[2], cdb[3]]);
                let count = u16::from_be_bytes([cdb[4], cdb[5]]);
                f.push(CdbField {
                    name: "Starting Element".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", start),
                    decoded: format!("element {}", start),
                });
                f.push(CdbField {
                    name: "Number of Elements".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", count),
                    decoded: format!("{}", count),
                });
            }
            if cdb.len() >= 10 {
                let alloc = ((cdb[7] as u32) << 16) | ((cdb[8] as u32) << 8) | (cdb[9] as u32);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:06X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 12 { f.push(control_byte_field(cdb, 11)); }
            f
        }
        // SEND VOLUME TAG
        0xB6 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = (cdb[1] >> 4) & 0x0F;
                f.push(CdbField {
                    name: "Send Action Code".into(),
                    byte_offset: 1,
                    bit_range: Some("7:4".into()),
                    hex_value: format!("{:X}", sa),
                    decoded: match sa {
                        0x0 => "Assert primary volume tag",
                        0x1 => "Assert alternate volume tag",
                        0x2 => "Replace primary volume tag",
                        0x3 => "Replace alternate volume tag",
                        0x4 => "Undefined primary volume tag",
                        0x5 => "Undefined alternate volume tag",
                        _ => "Reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 4 {
                let start = u16::from_be_bytes([cdb[2], cdb[3]]);
                f.push(CdbField {
                    name: "Starting Element".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", start),
                    decoded: format!("element {}", start),
                });
            }
            if cdb.len() >= 10 {
                let len = u16::from_be_bytes([cdb[8], cdb[9]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 8,
                    bit_range: None,
                    hex_value: format!("{:04X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 12 { f.push(control_byte_field(cdb, 11)); }
            f
        }
        // READ ELEMENT STATUS
        0xB8 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 12 {
                let voltag = (cdb[1] >> 4) & 0x01;
                let elem_type = cdb[1] & 0x0F;
                let start = u16::from_be_bytes([cdb[2], cdb[3]]);
                let count = u16::from_be_bytes([cdb[4], cdb[5]]);
                let curdata = (cdb[6] >> 1) & 0x01;
                let dvcid = cdb[6] & 0x01;
                let alloc_len = ((cdb[7] as u32) << 16) | ((cdb[8] as u32) << 8) | (cdb[9] as u32);
                f.push(CdbField {
                    name: "VolTag".into(),
                    byte_offset: 1,
                    bit_range: Some("4".into()),
                    hex_value: format!("{}", voltag),
                    decoded: if voltag != 0 { "Return volume tag" } else { "No volume tag" }.into(),
                });
                f.push(CdbField {
                    name: "Element Type".into(),
                    byte_offset: 1,
                    bit_range: Some("3:0".into()),
                    hex_value: format!("{:X}", elem_type),
                    decoded: element_type_name(elem_type).into(),
                });
                f.push(CdbField {
                    name: "Starting Address".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", start),
                    decoded: format!("element {}", start),
                });
                f.push(CdbField {
                    name: "Number of Elements".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", count),
                    decoded: format!("{}", count),
                });
                f.push(CdbField {
                    name: "CurData".into(),
                    byte_offset: 6,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", curdata),
                    decoded: if curdata != 0 { "Current data only" } else { "All data" }.into(),
                });
                f.push(CdbField {
                    name: "DVCID".into(),
                    byte_offset: 6,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", dvcid),
                    decoded: if dvcid != 0 { "Return device ID" } else { "No device ID" }.into(),
                });
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:06X}", alloc_len),
                    decoded: format!("{} bytes", alloc_len),
                });
                f.push(control_byte_field(cdb, 11));
            }
            f
        }
        // INIT ELEMENT STATUS WITH RANGE
        0xE7 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let fast = (cdb[1] >> 1) & 0x01;
                let range = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "FAST".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", fast),
                    decoded: if fast != 0 { "Fast scan" } else { "Normal scan" }.into(),
                });
                f.push(CdbField {
                    name: "RANGE".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", range),
                    decoded: if range != 0 { "Range specified" } else { "All elements" }.into(),
                });
            }
            if cdb.len() >= 6 {
                let start = u16::from_be_bytes([cdb[2], cdb[3]]);
                let count = u16::from_be_bytes([cdb[4], cdb[5]]);
                f.push(CdbField {
                    name: "Starting Element".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:04X}", start),
                    decoded: format!("element {}", start),
                });
                f.push(CdbField {
                    name: "Number of Elements".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:04X}", count),
                    decoded: format!("{}", count),
                });
            }
            if cdb.len() >= 10 { f.push(control_byte_field(cdb, 9)); }
            f
        }
        _ => generic_cdb_fields(cdb),
    }
}
