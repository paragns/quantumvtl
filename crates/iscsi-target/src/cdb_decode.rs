use serde::Serialize;

use crate::scsi_log::{opcode_name, scsi_status_name, DeviceType, ScsiLogEntry};

/// Structured breakdown of a CDB (Command Descriptor Block).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CdbBreakdown {
    pub opcode: u8,
    pub opcode_name: String,
    pub cdb_length: usize,
    pub fields: Vec<CdbField>,
    pub hex_dump: String,
}

/// One parsed field from a CDB.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CdbField {
    pub name: String,
    pub byte_offset: usize,
    pub bit_range: Option<String>,
    pub hex_value: String,
    pub decoded: String,
}

/// Structured breakdown of a SCSI response.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResponseBreakdown {
    pub status: u8,
    pub status_name: String,
    pub data_in_length: usize,
    pub data_in_hex: Option<String>,
    pub sense: Option<SenseBreakdown>,
}

/// Parsed sense data.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SenseBreakdown {
    pub sense_key: u8,
    pub sense_key_name: String,
    pub asc: u8,
    pub ascq: u8,
    pub asc_description: String,
    pub hex_dump: String,
}

/// Decode a CDB into a structured field breakdown.
pub fn decode_cdb(cdb: &[u8], device_type: DeviceType) -> CdbBreakdown {
    let opcode = cdb.first().copied().unwrap_or(0);
    let name = opcode_name(opcode, device_type).to_string();
    let hex_dump = hex_string(cdb);

    let fields = match device_type {
        DeviceType::MediaChanger => decode_smc_cdb(opcode, cdb),
        DeviceType::TapeDrive => decode_ssc_cdb(opcode, cdb),
    };

    CdbBreakdown {
        opcode,
        opcode_name: name,
        cdb_length: cdb.len(),
        fields,
        hex_dump,
    }
}

/// Decode a SCSI response from a log entry.
pub fn decode_response(entry: &ScsiLogEntry) -> ResponseBreakdown {
    let sense = if !entry.sense.is_empty() {
        Some(decode_sense(&entry.sense))
    } else {
        None
    };

    ResponseBreakdown {
        status: entry.status,
        status_name: scsi_status_name(entry.status).to_string(),
        data_in_length: entry.data_in_len,
        data_in_hex: entry.data_in.as_ref().map(|d| hex_string(d)),
        sense,
    }
}

/// Decode fixed-format sense data.
pub fn decode_sense(sense: &[u8]) -> SenseBreakdown {
    let sense_key = if sense.len() > 2 { sense[2] & 0x0F } else { 0 };
    let asc = if sense.len() > 12 { sense[12] } else { 0 };
    let ascq = if sense.len() > 13 { sense[13] } else { 0 };

    SenseBreakdown {
        sense_key,
        sense_key_name: sense_key_name(sense_key).to_string(),
        asc,
        ascq,
        asc_description: asc_description(asc, ascq).to_string(),
        hex_dump: hex_string(sense),
    }
}

// --- Common CDB field decoders ---

fn common_cdb_fields(opcode: u8, cdb: &[u8]) -> Option<Vec<CdbField>> {
    match opcode {
        // TEST UNIT READY — 6-byte, no additional fields
        0x00 => Some(vec![opcode_field(cdb)]),
        // REQUEST SENSE
        0x03 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                f.push(CdbField {
                    name: "Allocation Length".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[4]),
                    decoded: format!("{} bytes", cdb[4]),
                });
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
            Some(f)
        }
        // MODE SELECT(6)
        0x15 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                f.push(CdbField {
                    name: "Parameter List Length".into(),
                    byte_offset: 4,
                    bit_range: None,
                    hex_value: format!("{:02X}", cdb[4]),
                    decoded: format!("{} bytes", cdb[4]),
                });
            }
            Some(f)
        }
        // MODE SENSE(6)
        0x1A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 3 {
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: mode_page_name(page).into(),
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
            Some(f)
        }
        // LOG SENSE
        0x4D => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 3 {
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: format!("Log page 0x{:02X}", page),
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
            Some(f)
        }
        // MODE SELECT(10)
        0x55 => {
            let mut f = vec![opcode_field(cdb)];
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
            Some(f)
        }
        // MODE SENSE(10)
        0x5A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 3 {
                let page = cdb[2] & 0x3F;
                f.push(CdbField {
                    name: "Page Code".into(),
                    byte_offset: 2,
                    bit_range: Some("5:0".into()),
                    hex_value: format!("{:02X}", page),
                    decoded: mode_page_name(page).into(),
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
            Some(f)
        }
        // REPORT LUNS
        0xA0 => {
            let mut f = vec![opcode_field(cdb)];
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
            Some(f)
        }
        _ => None,
    }
}

fn decode_smc_cdb(opcode: u8, cdb: &[u8]) -> Vec<CdbField> {
    if let Some(f) = common_cdb_fields(opcode, cdb) {
        return f;
    }
    match opcode {
        // INITIALIZE ELEMENT STATUS
        0x07 => vec![opcode_field(cdb)],
        // MOVE MEDIUM
        0xA5 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 12 {
                let transport = u16::from_be_bytes([cdb[2], cdb[3]]);
                let source = u16::from_be_bytes([cdb[4], cdb[5]]);
                let dest = u16::from_be_bytes([cdb[6], cdb[7]]);
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
            }
            f
        }
        // READ ELEMENT STATUS
        0xB8 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 12 {
                let elem_type = (cdb[1] >> 4) & 0x0F;
                let start = u16::from_be_bytes([cdb[2], cdb[3]]);
                let count = u16::from_be_bytes([cdb[4], cdb[5]]);
                let alloc_len = ((cdb[7] as u32) << 16) | ((cdb[8] as u32) << 8) | (cdb[9] as u32);
                f.push(CdbField {
                    name: "Element Type".into(),
                    byte_offset: 1,
                    bit_range: Some("7:4".into()),
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
                    name: "Allocation Length".into(),
                    byte_offset: 7,
                    bit_range: None,
                    hex_value: format!("{:06X}", alloc_len),
                    decoded: format!("{} bytes", alloc_len),
                });
            }
            f
        }
        _ => generic_cdb_fields(cdb),
    }
}

fn decode_ssc_cdb(opcode: u8, cdb: &[u8]) -> Vec<CdbField> {
    if let Some(f) = common_cdb_fields(opcode, cdb) {
        return f;
    }
    match opcode {
        // REWIND
        0x01 => vec![opcode_field(cdb)],
        // READ BLOCK LIMITS
        0x05 => vec![opcode_field(cdb)],
        // READ(6)
        0x08 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let fixed = (cdb[1] & 0x01) != 0;
                let transfer_len =
                    ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "Fixed".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", u8::from(fixed)),
                    decoded: if fixed {
                        "Fixed-length blocks"
                    } else {
                        "Variable-length"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Transfer Length".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", transfer_len),
                    decoded: if fixed {
                        format!("{} blocks", transfer_len)
                    } else {
                        format!("{} bytes", transfer_len)
                    },
                });
            }
            f
        }
        // WRITE(6)
        0x0A => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let fixed = (cdb[1] & 0x01) != 0;
                let transfer_len =
                    ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                f.push(CdbField {
                    name: "Fixed".into(),
                    byte_offset: 1,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", u8::from(fixed)),
                    decoded: if fixed {
                        "Fixed-length blocks"
                    } else {
                        "Variable-length"
                    }
                    .into(),
                });
                f.push(CdbField {
                    name: "Transfer Length".into(),
                    byte_offset: 2,
                    bit_range: None,
                    hex_value: format!("{:06X}", transfer_len),
                    decoded: if fixed {
                        format!("{} blocks", transfer_len)
                    } else {
                        format!("{} bytes", transfer_len)
                    },
                });
            }
            f
        }
        // WRITE FILEMARKS(6)
        0x10 => {
            let mut f = vec![opcode_field(cdb)];
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
            f
        }
        // SPACE(6)
        0x11 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let code = cdb[1] & 0x07;
                let count_raw = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
                // Sign-extend 24-bit value
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
            f
        }
        // LOAD UNLOAD
        0x1B => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 5 {
                let load = (cdb[4] & 0x01) != 0;
                f.push(CdbField {
                    name: "Load".into(),
                    byte_offset: 4,
                    bit_range: Some("0".into()),
                    hex_value: format!("{}", u8::from(load)),
                    decoded: if load { "Load" } else { "Unload" }.into(),
                });
            }
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
                        0x00 => "Short form",
                        0x01 => "Long form",
                        0x06 => "Extended form",
                        _ => "Unknown",
                    }
                    .into(),
                });
            }
            f
        }
        // REPORT DENSITY SUPPORT
        0x44 => {
            let mut f = vec![opcode_field(cdb)];
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
            f
        }
        // SPACE(16)
        0x91 => {
            let mut f = vec![opcode_field(cdb)];
            if cdb.len() >= 16 {
                let code = cdb[1] & 0x07;
                f.push(CdbField {
                    name: "Code".into(),
                    byte_offset: 1,
                    bit_range: Some("2:0".into()),
                    hex_value: format!("{:X}", code),
                    decoded: space_code_name(code).into(),
                });
            }
            f
        }
        // LOCATE(10)
        0x2B => {
            let mut f = vec![opcode_field(cdb)];
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
            f
        }
        _ => generic_cdb_fields(cdb),
    }
}

fn generic_cdb_fields(cdb: &[u8]) -> Vec<CdbField> {
    vec![
        opcode_field(cdb),
        CdbField {
            name: "Raw Bytes".into(),
            byte_offset: 0,
            bit_range: None,
            hex_value: hex_string(cdb),
            decoded: format!("{} byte CDB", cdb.len()),
        },
    ]
}

fn opcode_field(cdb: &[u8]) -> CdbField {
    let op = cdb.first().copied().unwrap_or(0);
    CdbField {
        name: "Opcode".into(),
        byte_offset: 0,
        bit_range: None,
        hex_value: format!("{:02X}", op),
        decoded: format!("0x{:02X}", op),
    }
}

// --- Name lookup helpers ---

fn sense_key_name(key: u8) -> &'static str {
    match key {
        0x0 => "NO SENSE",
        0x1 => "RECOVERED ERROR",
        0x2 => "NOT READY",
        0x3 => "MEDIUM ERROR",
        0x4 => "HARDWARE ERROR",
        0x5 => "ILLEGAL REQUEST",
        0x6 => "UNIT ATTENTION",
        0x7 => "DATA PROTECT",
        0x8 => "BLANK CHECK",
        0xB => "ABORTED COMMAND",
        0xD => "VOLUME OVERFLOW",
        0xE => "MISCOMPARE",
        _ => "UNKNOWN",
    }
}

fn asc_description(asc: u8, ascq: u8) -> &'static str {
    match (asc, ascq) {
        (0x00, 0x00) => "No additional sense information",
        (0x04, 0x00) => "Logical unit not ready, cause not reportable",
        (0x04, 0x01) => "Logical unit is in process of becoming ready",
        (0x04, 0x02) => "Logical unit not ready, initializing required",
        (0x04, 0x03) => "Logical unit not ready, manual intervention required",
        (0x14, 0x00) => "Record not found",
        (0x20, 0x00) => "Invalid command operation code",
        (0x21, 0x00) => "Logical block address out of range",
        (0x24, 0x00) => "Invalid field in CDB",
        (0x25, 0x00) => "Logical unit not supported",
        (0x26, 0x00) => "Invalid field in parameter list",
        (0x27, 0x00) => "Write protected",
        (0x28, 0x00) => "Not ready to ready change, medium may have changed",
        (0x29, 0x00) => "Power on, reset, or bus device reset occurred",
        (0x2A, 0x01) => "Mode parameters changed",
        (0x3A, 0x00) => "Medium not present",
        (0x3B, 0x0D) => "Medium destination element full",
        (0x3B, 0x0E) => "Medium source element empty",
        (0x44, 0x00) => "Internal target failure",
        _ => "Unknown ASC/ASCQ",
    }
}

fn vpd_page_name(page: u8) -> &'static str {
    match page {
        0x00 => "Supported VPD Pages",
        0x80 => "Unit Serial Number",
        0x83 => "Device Identification",
        0x86 => "Extended INQUIRY Data",
        0xB0 => "Block Limits",
        0xB1 => "Block Device Characteristics",
        _ => "Unknown VPD page",
    }
}

fn mode_page_name(page: u8) -> &'static str {
    match page {
        0x00 => "Vendor Specific",
        0x01 => "Read-Write Error Recovery",
        0x02 => "Disconnect-Reconnect",
        0x0A => "Control",
        0x0F => "Data Compression",
        0x10 => "Device Configuration",
        0x11 => "Medium Partition(1)",
        0x1C => "Informational Exceptions Control",
        0x1D => "Element Address Assignment",
        0x1E => "Transport Geometry",
        0x1F => "Device Capabilities",
        0x3F => "All Pages",
        _ => "Unknown page",
    }
}

fn element_type_name(etype: u8) -> &'static str {
    match etype {
        0x0 => "All Types",
        0x1 => "Medium Transport",
        0x2 => "Storage",
        0x3 => "Import/Export",
        0x4 => "Data Transfer",
        _ => "Unknown",
    }
}

fn space_code_name(code: u8) -> &'static str {
    match code {
        0 => "Blocks",
        1 => "Filemarks",
        3 => "End-of-data",
        _ => "Unknown",
    }
}

fn hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_inquiry_cdb() {
        let cdb = [0x12, 0x00, 0x00, 0x00, 0x60, 0x00];
        let bd = decode_cdb(&cdb, DeviceType::MediaChanger);
        assert_eq!(bd.opcode, 0x12);
        assert_eq!(bd.opcode_name, "INQUIRY");
        assert!(bd.fields.iter().any(|f| f.name == "EVPD"));
        assert!(bd.fields.iter().any(|f| f.name == "Allocation Length"));
    }

    #[test]
    fn decode_move_medium_cdb() {
        let cdb = [
            0xA5, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let bd = decode_cdb(&cdb, DeviceType::MediaChanger);
        assert_eq!(bd.opcode_name, "MOVE MEDIUM");
        assert!(bd.fields.iter().any(|f| f.name == "Source Address"));
        assert!(bd.fields.iter().any(|f| f.name == "Destination Address"));
    }

    #[test]
    fn decode_read6_cdb() {
        let cdb = [0x08, 0x00, 0x00, 0x01, 0x00, 0x00];
        let bd = decode_cdb(&cdb, DeviceType::TapeDrive);
        assert_eq!(bd.opcode_name, "READ(6)");
        assert!(bd.fields.iter().any(|f| f.name == "Fixed"));
        assert!(bd.fields.iter().any(|f| f.name == "Transfer Length"));
    }

    #[test]
    fn decode_sense_data() {
        // Fixed-format sense: sense key 0x05 (ILLEGAL REQUEST), ASC 0x24, ASCQ 0x00
        let sense = [
            0x70, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00,
        ];
        let sb = decode_sense(&sense);
        assert_eq!(sb.sense_key, 0x05);
        assert_eq!(sb.sense_key_name, "ILLEGAL REQUEST");
        assert_eq!(sb.asc, 0x24);
        assert_eq!(sb.asc_description, "Invalid field in CDB");
    }
}
