//! Common CDB decoders shared between SSC and SMC device types.

use super::names::{mode_page_name, vpd_page_name, log_page_name};
use super::{control_byte_field, opcode_field, CdbField};

/// Try to decode a CDB that is common to both SSC and SMC (SPC commands).
/// Returns None if the opcode is device-type-specific.
pub fn common_cdb_fields(opcode: u8, cdb: &[u8]) -> Option<Vec<CdbField>> {
    match opcode {
        // TEST UNIT READY
        0x00 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // REQUEST SENSE
        0x03 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let desc = (cdb[1] >> 0) & 0x01;
                f.push(CdbField {
                    name: "DESC".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", desc),
                    decoded: if desc != 0 {
                        "Descriptor format"
                    } else {
                        "Fixed format"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 5 {
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[4]),
                    decoded: format!("{} bytes", cdb[4]),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // INQUIRY
        0x12 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let evpd = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "EVPD".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{:02X}", evpd),
                    decoded: if evpd != 0 {
                        "VPD page requested"
                    } else {
                        "Standard inquiry"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[2]),
                    decoded: vpd_page_name(cdb[2]).into(),
                });
            }
            if cdb.len() >= 5 {
                let alloc = u16::from_be_bytes([cdb[3], cdb[4]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:04X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // MODE SELECT(6)
        0x15 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let pf = (cdb[1] >> 4) & 0x01;
                let sp = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "PF".into(),
                    byte_offset: 1,
                    bit_range: Some("4".into()),
                    hex_value: format!("{}", pf),
                    decoded: if pf != 0 {
                        "Page format"
                    } else {
                        "Vendor specific"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "SP".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", sp),
                    decoded: if sp != 0 {
                        "Save pages"
                    } else {
                        "Don't save"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 5 {
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[4]),
                    decoded: format!("{} bytes", cdb[4]),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // MODE SENSE(6)
        0x1A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let dbd = (cdb[1] >> 3) & 0x01;
                f.push(CdbField {
                    name: "DBD".into(),
                    byte_offset: 1,
                    bit_range: Some("3".into()),
                    hex_value: format!("{}", dbd),
                    decoded: if dbd != 0 {
                        "Disable block descriptors"
                    } else {
                        "Return block descriptors"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                let pc = (cdb[2] >> 6) & 0x03;
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "PC".into(),
                    byte_offset: 2,
                    bit_range: Some("7:6".into()),
                    hex_value: format!("{}", pc),
                    decoded: match pc {
                        0 => "Current values",
                        1 => "Changeable values",
                        2 => "Default values",
                        3 => "Saved values",
                        _ => "Unknown",
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: mode_page_name(page).into(),
                });
            }
            if cdb.len() >= 4 {
                f.push(CdbField {
                    name: "Subpage Code".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[3]),
                    decoded: format!("0x{:02X}", cdb[3]),
                });
            }
            if cdb.len() >= 5 {
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[4]),
                    decoded: format!("{} bytes", cdb[4]),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // SEND DIAGNOSTIC
        0x1D => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let self_test = (cdb[1] >> 2) & 0x01;
                let pf = (cdb[1] >> 4) & 0x01;
                let unit_ol = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "SELF-TEST".into(),
                    byte_offset: 1,
                    bit_range: Some("2".into()),
                    hex_value: format!("{}", self_test),
                    decoded: if self_test != 0 {
                        "Run self-test"
                    } else {
                        "No self-test"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "PF".into(),
                    byte_offset: 1,
                    bit_range: Some("4".into()),
                    hex_value: format!("{}", pf),
                    decoded: if pf != 0 {
                        "Page format"
                    } else {
                        "Vendor specific"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "UNITOFFL".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", unit_ol),
                    decoded: if unit_ol != 0 {
                        "Unit offline"
                    } else {
                        "No offline"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 5 {
                let len = u16::from_be_bytes([cdb[3], cdb[4]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:04X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // PREVENT ALLOW MEDIUM REMOVAL
        0x1E => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let prevent = cdb[4] & 0x03;
                f.push(CdbField {
                    name: "Prevent".into(),
                    byte_offset: 4,
                    bit_range: Some("1:0".into()),
                    hex_value: format!("{:02X}", prevent),
                    decoded: match prevent {
                        0 => "Allow removal",
                        1 => "Prevent removal",
                        2 => "Allow (persistent)",
                        3 => "Prevent (persistent)",
                        _ => "Unknown",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 6 {
                f.push(control_byte_field(cdb, 5));
            }
            Some(f)
        }
        // READ BUFFER
        0x3C => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let mode = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Mode".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", mode),
                    decoded: match mode {
                        0x00 => "Combined header and data",
                        0x02 => "Data",
                        0x03 => "Descriptor",
                        0x0A => "Read data from echo buffer",
                        0x0B => "Echo buffer descriptor",
                        0x1C => "Error history",
                        _ => "Vendor/reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                f.push(CdbField {
                    name: "Buffer ID".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[2]),
                    decoded: format!("{}", cdb[2]),
                });
            }
            if cdb.len() >= 6 {
                let offset = ((cdb[3] as u32) << 16) | ((cdb[4] as u32) << 8) | (cdb[5] as u32);
                f.push(CdbField {
                    name: "Buffer Offset".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:06X}", offset),
                    decoded: format!("{}", offset),
                });
            }
            if cdb.len() >= 9 {
                let alloc =
                    ((cdb[6] as u32) << 16) | ((cdb[7] as u32) << 8) | (cdb[8] as u32);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:06X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // WRITE BUFFER
        0x3B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let mode = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Mode".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", mode),
                    decoded: match mode {
                        0x00 => "Combined header and data",
                        0x02 => "Data",
                        0x04 => "Download microcode",
                        0x05 => "Download microcode and save",
                        0x06 => "Download microcode with offsets",
                        0x07 => "Download microcode with offsets and save",
                        0x0A => "Write data to echo buffer",
                        _ => "Vendor/reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                f.push(CdbField {
                    name: "Buffer ID".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[2]),
                    decoded: format!("{}", cdb[2]),
                });
            }
            if cdb.len() >= 6 {
                let offset = ((cdb[3] as u32) << 16) | ((cdb[4] as u32) << 8) | (cdb[5] as u32);
                f.push(CdbField {
                    name: "Buffer Offset".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:06X}", offset),
                    decoded: format!("{}", offset),
                });
            }
            if cdb.len() >= 9 {
                let len =
                    ((cdb[6] as u32) << 16) | ((cdb[7] as u32) << 8) | (cdb[8] as u32);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:06X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // LOG SENSE
        0x4D => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sp = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "SP".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", sp),
                    decoded: if sp != 0 {
                        "Save parameters"
                    } else {
                        "Don't save"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                let pc = (cdb[2] >> 6) & 0x03;
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "PC".into(),
                    byte_offset: 2,
                    bit_range: Some("7:6".into()),
                    hex_value: format!("{}", pc),
                    decoded: match pc {
                        0 => "Threshold values",
                        1 => "Cumulative values",
                        2 => "Default threshold values",
                        3 => "Default cumulative values",
                        _ => "Unknown",
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: log_page_name(page).into(),
                });
            }
            if cdb.len() >= 4 {
                f.push(CdbField {
                    name: "Subpage Code".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[3]),
                    decoded: format!("0x{:02X}", cdb[3]),
                });
            }
            if cdb.len() >= 7 {
                let param_ptr = u16::from_be_bytes([cdb[5], cdb[6]]);
                f.push(CdbField {
                    name: "Parameter Pointer".into(),
                    byte_offset: 5,
                    bit_range: None,
                    hex_value: format!("{:04X}", param_ptr),
                    decoded: format!("0x{:04X}", param_ptr),
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
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // LOG SELECT
        0x4C => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sp = cdb[1] & 0x01;
                let pcr = (cdb[1] >> 1) & 0x01;
                f.push(CdbField {
                    name: "SP".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", sp),
                    decoded: if sp != 0 {
                        "Save parameters"
                    } else {
                        "Don't save"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "PCR".into(),
                    byte_offset: 1,
                    bit_range: Some("1".into()),
                    hex_value: format!("{}", pcr),
                    decoded: if pcr != 0 {
                        "Parameter code reset"
                    } else {
                        "No reset"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                let pc = (cdb[2] >> 6) & 0x03;
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "PC".into(),
                    byte_offset: 2,
                    bit_range: Some("7:6".into()),
                    hex_value: format!("{}", pc),
                    decoded: match pc {
                        0 => "Threshold values",
                        1 => "Cumulative values",
                        2 => "Default threshold values",
                        3 => "Default cumulative values",
                        _ => "Unknown",
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: log_page_name(page).into(),
                });
            }
            if cdb.len() >= 9 {
                let len = u16::from_be_bytes([cdb[7], cdb[8]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:04X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // MODE SELECT(10)
        0x55 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let pf = (cdb[1] >> 4) & 0x01;
                let sp = cdb[1] & 0x01;
                f.push(CdbField {
                    name: "PF".into(),
                    byte_offset: 1,
                    bit_range: Some("4".into()),
                    hex_value: format!("{}", pf),
                    decoded: if pf != 0 {
                        "Page format"
                    } else {
                        "Vendor specific"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "SP".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", sp),
                    decoded: if sp != 0 {
                        "Save pages"
                    } else {
                        "Don't save"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 9 {
                let len = u16::from_be_bytes([cdb[7], cdb[8]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:04X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // MODE SENSE(10)
        0x5A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let llbaa = (cdb[1] >> 4) & 0x01;
                let dbd = (cdb[1] >> 3) & 0x01;
                f.push(CdbField {
                    name: "LLBAA".into(),
                    byte_offset: 1,
                    bit_range: Some("4".into()),
                    hex_value: format!("{}", llbaa),
                    decoded: if llbaa != 0 {
                        "Long LBA accepted"
                    } else {
                        "Short LBA only"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "DBD".into(),
                    byte_offset: 1,
                    bit_range: Some("3".into()),
                    hex_value: format!("{}", dbd),
                    decoded: if dbd != 0 {
                        "Disable block descriptors"
                    } else {
                        "Return block descriptors"
                    }
                    .into(),
                });
            }
            if cdb.len() >= 3 {
                let pc = (cdb[2] >> 6) & 0x03;
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "PC".into(),
                    byte_offset: 2,
                    bit_range: Some("7:6".into()),
                    hex_value: format!("{}", pc),
                    decoded: match pc {
                        0 => "Current values",
                        1 => "Changeable values",
                        2 => "Default values",
                        3 => "Saved values",
                        _ => "Unknown",
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: mode_page_name(page).into(),
                });
            }
            if cdb.len() >= 4 {
                f.push(CdbField {
                    name: "Subpage Code".into(),
                    byte_offset: 3,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[3]),
                    decoded: format!("0x{:02X}", cdb[3]),
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
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // PERSISTENT RESERVE IN
        0x5E => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", sa),
                    decoded: match sa {
                        0x00 => "Read keys",
                        0x01 => "Read reservation",
                        0x02 => "Report capabilities",
                        0x03 => "Read full status",
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
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // PERSISTENT RESERVE OUT
        0x5F => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", sa),
                    decoded: match sa {
                        0x00 => "Register",
                        0x01 => "Reserve",
                        0x02 => "Release",
                        0x03 => "Clear",
                        0x04 => "Preempt",
                        0x05 => "Preempt and abort",
                        0x06 => "Register and ignore existing key",
                        0x07 => "Register and move",
                        _ => "Unknown",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 6 {
                let scope_type = if cdb.len() >= 3 { cdb[2] } else { 0 };
                let scope = (scope_type >> 4) & 0x0F;
                let rtype = scope_type & 0x0F;
                f.push(CdbField {
                    name: "Scope".into(),
                    byte_offset: 2,
                    bit_range: Some("7:4".into()),
                    hex_value: format!("{:X}", scope),
                    decoded: match scope {
                        0 => "LU scope".into(),
                        _ => format!("Scope {}", scope),
                    },
                });
                f.push(CdbField {
                    name: "Type".into(),
                    byte_offset: 2,
                    bit_range: Some("3:0".into()),
                    hex_value: format!("{:X}", rtype),
                    decoded: match rtype {
                        1 => "Write exclusive",
                        3 => "Exclusive access",
                        5 => "Write exclusive - registrants only",
                        6 => "Exclusive access - registrants only",
                        7 => "Write exclusive - all registrants",
                        8 => "Exclusive access - all registrants",
                        _ => "Reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 9 {
                let len = u32::from_be_bytes([cdb[5], cdb[6], cdb[7], cdb[8]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 5,
                    bit_range: None,
                    hex_value: format!("{:08X}", len),
                    decoded: format!("{} bytes", len),
                });
            }
            if cdb.len() >= 10 {
                f.push(control_byte_field(cdb, 9));
            }
            Some(f)
        }
        // REPORT LUNS
        0xA0 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 3 {
                f.push(CdbField {
                    name: "Select Report".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[2]),
                    decoded: match cdb[2] {
                        0x00 => "Logical units",
                        0x01 => "Well known logical units",
                        0x02 => "All",
                        _ => "Reserved",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 10 {
                let alloc = u32::from_be_bytes([cdb[6], cdb[7], cdb[8], cdb[9]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:08X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
            }
            if cdb.len() >= 12 {
                f.push(control_byte_field(cdb, 11));
            }
            Some(f)
        }
        // MAINTENANCE IN
        0xA3 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", sa),
                    decoded: match sa {
                        0x0A => "Report target port groups",
                        0x0C => "Report supported operation codes",
                        0x0D => "Report supported task management functions",
                        _ => "Unknown/vendor",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 12 {
                let alloc = u32::from_be_bytes([cdb[6], cdb[7], cdb[8], cdb[9]]);
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:08X}", alloc),
                    decoded: format!("{} bytes", alloc),
                });
                f.push(control_byte_field(cdb, 11));
            }
            Some(f)
        }
        // MAINTENANCE OUT
        0xA4 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 2 {
                let sa = cdb[1] & 0x1F;
                f.push(CdbField {
                    name: "Service Action".into(),
                    byte_offset: 1,
                    bit_range: Some("4:0".into()),
                    hex_value: format!("{:02X}", sa),
                    decoded: match sa {
                        0x0A => "Set target port groups",
                        _ => "Unknown/vendor",
                    }
                    .into(),
                });
            }
            if cdb.len() >= 12 {
                let len = u32::from_be_bytes([cdb[6], cdb[7], cdb[8], cdb[9]]);
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 6,
                    bit_range: None,
                    hex_value: format!("{:08X}", len),
                    decoded: format!("{} bytes", len),
                });
                f.push(control_byte_field(cdb, 11));
            }
            Some(f)
        }
        _ => None,
    }
}
