pub mod names;
mod cdb_common;
mod cdb_ssc;
mod cdb_smc;
pub mod sense;
pub mod response;

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

/// Nestable data field for exhaustive response/sense decoding.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DataField {
    pub name: String,
    pub byte_offset: usize,
    pub bit_range: Option<String>,
    pub hex_value: String,
    pub decoded: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "utoipa", schema(no_recursion))]
    pub children: Option<Vec<DataField>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_in_fields: Option<Vec<DataField>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<DataField>>,
}

/// Decode a CDB into a structured field breakdown.
pub fn decode_cdb(cdb: &[u8], device_type: DeviceType) -> CdbBreakdown {
    let opcode = cdb.first().copied().unwrap_or(0);
    let name = opcode_name(opcode, device_type).to_string();
    let hex_dump = hex_string(cdb);

    let fields = match device_type {
        DeviceType::MediaChanger => cdb_smc::decode_smc_cdb(opcode, cdb),
        DeviceType::TapeDrive => cdb_ssc::decode_ssc_cdb(opcode, cdb),
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
        Some(sense::decode_sense(&entry.sense))
    } else {
        None
    };

    let data_in_fields = entry
        .data_in
        .as_ref()
        .and_then(|data| response::decode_response_data(entry.opcode, &entry.cdb, data));

    ResponseBreakdown {
        status: entry.status,
        status_name: scsi_status_name(entry.status).to_string(),
        data_in_length: entry.data_in_len,
        data_in_hex: entry.data_in.as_ref().map(|d| hex_string(d)),
        sense,
        data_in_fields,
    }
}

// --- Helpers used across submodules ---

pub(crate) fn hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn opcode_field(cdb: &[u8]) -> CdbField {
    let op = cdb.first().copied().unwrap_or(0);
    CdbField {
        name: "Opcode".into(),
        byte_offset: 0,
        bit_range: None,
        hex_value: format!("{:02X}", op),
        decoded: format!("0x{:02X}", op),
    }
}

pub(crate) fn control_byte_field(cdb: &[u8], offset: usize) -> CdbField {
    let val = cdb.get(offset).copied().unwrap_or(0);
    let naca = (val >> 2) & 0x01;
    let decoded = if naca != 0 {
        format!("NACA=1, vendor={:02b}", (val >> 6) & 0x03)
    } else if val == 0 {
        "0x00".into()
    } else {
        format!("0x{:02X}", val)
    };
    CdbField {
        name: "Control".into(),
        byte_offset: offset,
        bit_range: None,
        hex_value: format!("{:02X}", val),
        decoded,
    }
}

pub(crate) fn generic_cdb_fields(cdb: &[u8]) -> Vec<CdbField> {
    let mut f = vec![opcode_field(cdb)];
    if cdb.len() > 1 {
        f.push(CdbField {
            name: "Raw Bytes".into(),
            byte_offset: 0,
            bit_range: None,
            hex_value: hex_string(cdb),
            decoded: format!("{} byte CDB", cdb.len()),
        });
    }
    let ctrl_offset = match cdb.len() {
        6 => Some(5),
        10 => Some(9),
        12 => Some(11),
        16 => Some(15),
        _ => None,
    };
    if let Some(off) = ctrl_offset {
        f.push(control_byte_field(cdb, off));
    }
    f
}

/// Helper to make a DataField leaf (no children).
pub(crate) fn data_field(
    name: impl Into<String>,
    byte_offset: usize,
    bit_range: Option<&str>,
    hex_value: impl Into<String>,
    decoded: impl Into<String>,
) -> DataField {
    DataField {
        name: name.into(),
        byte_offset,
        bit_range: bit_range.map(|s| s.into()),
        hex_value: hex_value.into(),
        decoded: decoded.into(),
        children: None,
    }
}

/// Helper to make a DataField with children.
pub(crate) fn data_field_parent(
    name: impl Into<String>,
    byte_offset: usize,
    bit_range: Option<&str>,
    hex_value: impl Into<String>,
    decoded: impl Into<String>,
    children: Vec<DataField>,
) -> DataField {
    DataField {
        name: name.into(),
        byte_offset,
        bit_range: bit_range.map(|s| s.into()),
        hex_value: hex_value.into(),
        decoded: decoded.into(),
        children: Some(children),
    }
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
        let sense = [
            0x70, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x24, 0x00,
        ];
        let sb = sense::decode_sense(&sense);
        assert_eq!(sb.sense_key, 0x05);
        assert_eq!(sb.sense_key_name, "ILLEGAL REQUEST");
        assert_eq!(sb.asc, 0x24);
        assert_eq!(sb.asc_description, "Invalid field in CDB");
    }

    #[test]
    fn response_breakdown_has_data_in_fields() {
        let rb = ResponseBreakdown {
            status: 0,
            status_name: "GOOD".into(),
            data_in_length: 0,
            data_in_hex: None,
            sense: None,
            data_in_fields: None,
        };
        let json = serde_json::to_string(&rb).unwrap();
        assert!(!json.contains("data_in_fields"));
    }

    #[test]
    fn data_field_serialization() {
        let field = DataField {
            name: "Test".into(),
            byte_offset: 0,
            bit_range: Some("7:4".into()),
            hex_value: "F0".into(),
            decoded: "Test value".into(),
            children: Some(vec![data_field("Child", 1, None, "01", "child val")]),
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("children"));
        assert!(json.contains("Child"));
    }
}
