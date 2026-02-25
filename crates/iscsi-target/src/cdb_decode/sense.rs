//! Exhaustive fixed-format sense data parser.

use super::names::{asc_description, response_code_name, sense_key_name};
use super::{data_field, hex_string, DataField, SenseBreakdown};

/// Decode fixed-format sense data into a SenseBreakdown with full field breakdown.
pub fn decode_sense(sense: &[u8]) -> SenseBreakdown {
    let sense_key = if sense.len() > 2 { sense[2] & 0x0F } else { 0 };
    let asc = if sense.len() > 12 { sense[12] } else { 0 };
    let ascq = if sense.len() > 13 { sense[13] } else { 0 };

    let fields = decode_sense_fields(sense);

    SenseBreakdown {
        sense_key,
        sense_key_name: sense_key_name(sense_key).to_string(),
        asc,
        ascq,
        asc_description: asc_description(asc, ascq).to_string(),
        hex_dump: hex_string(sense),
        fields: Some(fields),
    }
}

fn decode_sense_fields(sense: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if sense.is_empty() {
        return fields;
    }

    // Byte 0: VALID (bit 7) + RESPONSE CODE (bits 6:0)
    let b0 = sense[0];
    let valid = (b0 >> 7) & 1;
    let response_code = b0 & 0x7F;
    fields.push(data_field(
        "VALID",
        0,
        Some("7"),
        format!("{}", valid),
        if valid != 0 {
            "Information field is valid"
        } else {
            "Information field is not valid"
        },
    ));
    fields.push(data_field(
        "RESPONSE CODE",
        0,
        Some("6:0"),
        format!("{:02X}", response_code),
        response_code_name(response_code),
    ));

    if sense.len() < 3 {
        return fields;
    }

    // Byte 1: Obsolete
    fields.push(data_field(
        "Obsolete",
        1,
        None,
        format!("{:02X}", sense[1]),
        format!("0x{:02X}", sense[1]),
    ));

    // Byte 2: FILEMARK (7), EOM (6), ILI (5), SDAT_OVFL (4), SENSE KEY (3:0)
    let b2 = sense[2];
    let filemark = (b2 >> 7) & 1;
    let eom = (b2 >> 6) & 1;
    let ili = (b2 >> 5) & 1;
    let sdat_ovfl = (b2 >> 4) & 1;
    let sk = b2 & 0x0F;

    fields.push(data_field(
        "FILEMARK",
        2,
        Some("7"),
        format!("{}", filemark),
        if filemark != 0 {
            "Filemark detected"
        } else {
            "No filemark"
        },
    ));
    fields.push(data_field(
        "EOM",
        2,
        Some("6"),
        format!("{}", eom),
        if eom != 0 {
            "End-of-medium"
        } else {
            "Not at end-of-medium"
        },
    ));
    fields.push(data_field(
        "ILI",
        2,
        Some("5"),
        format!("{}", ili),
        if ili != 0 {
            "Incorrect length indicator"
        } else {
            "Length OK"
        },
    ));
    fields.push(data_field(
        "SDAT_OVFL",
        2,
        Some("4"),
        format!("{}", sdat_ovfl),
        if sdat_ovfl != 0 {
            "Sense data overflow"
        } else {
            "No overflow"
        },
    ));
    fields.push(data_field(
        "SENSE KEY",
        2,
        Some("3:0"),
        format!("{:X}", sk),
        sense_key_name(sk),
    ));

    // Bytes 3-6: INFORMATION
    if sense.len() >= 7 {
        let info = u32::from_be_bytes([sense[3], sense[4], sense[5], sense[6]]);
        let decoded = if valid != 0 {
            format!("{} (0x{:08X})", info, info)
        } else {
            format!("0x{:08X} (VALID=0, may not be meaningful)", info)
        };
        fields.push(data_field(
            "INFORMATION",
            3,
            None,
            format!(
                "{:02X} {:02X} {:02X} {:02X}",
                sense[3], sense[4], sense[5], sense[6]
            ),
            decoded,
        ));
    }

    // Byte 7: ADDITIONAL SENSE LENGTH
    if sense.len() >= 8 {
        fields.push(data_field(
            "ADDITIONAL SENSE LENGTH",
            7,
            None,
            format!("{:02X}", sense[7]),
            format!("{} bytes follow", sense[7]),
        ));
    }

    // Bytes 8-11: COMMAND-SPECIFIC INFORMATION
    if sense.len() >= 12 {
        let cmd_info = u32::from_be_bytes([sense[8], sense[9], sense[10], sense[11]]);
        fields.push(data_field(
            "COMMAND-SPECIFIC INFORMATION",
            8,
            None,
            format!(
                "{:02X} {:02X} {:02X} {:02X}",
                sense[8], sense[9], sense[10], sense[11]
            ),
            format!("0x{:08X}", cmd_info),
        ));
    }

    // Bytes 12-13: ASC / ASCQ
    if sense.len() >= 13 {
        fields.push(data_field(
            "ASC",
            12,
            None,
            format!("{:02X}", sense[12]),
            format!("Additional Sense Code 0x{:02X}", sense[12]),
        ));
    }
    if sense.len() >= 14 {
        fields.push(data_field(
            "ASCQ",
            13,
            None,
            format!("{:02X}", sense[13]),
            asc_description(sense[12], sense[13]),
        ));
    }

    // Byte 14: FIELD REPLACEABLE UNIT CODE
    if sense.len() >= 15 {
        fields.push(data_field(
            "FRU CODE",
            14,
            None,
            format!("{:02X}", sense[14]),
            if sense[14] == 0 {
                "No FRU code".into()
            } else {
                format!("FRU 0x{:02X}", sense[14])
            },
        ));
    }

    // Bytes 15-17: SENSE KEY SPECIFIC
    if sense.len() >= 18 {
        let sksv = (sense[15] >> 7) & 1;
        if sksv != 0 {
            decode_sense_key_specific(sk, &sense[15..18], &mut fields);
        } else {
            fields.push(data_field(
                "SENSE KEY SPECIFIC",
                15,
                None,
                format!("{:02X} {:02X} {:02X}", sense[15], sense[16], sense[17]),
                "SKSV=0, sense key specific data not valid",
            ));
        }
    }

    // Any additional sense bytes beyond 17
    if sense.len() > 18 {
        fields.push(data_field(
            "Additional Sense Bytes",
            18,
            None,
            hex_string(&sense[18..]),
            format!("{} additional bytes", sense.len() - 18),
        ));
    }

    fields
}

fn decode_sense_key_specific(sense_key: u8, sks: &[u8], fields: &mut Vec<DataField>) {
    match sense_key {
        // ILLEGAL REQUEST: field pointer
        0x05 => {
            let cd = (sks[0] >> 6) & 1;
            let bpv = (sks[0] >> 3) & 1;
            let bit_pointer = sks[0] & 0x07;
            let field_pointer = u16::from_be_bytes([sks[1], sks[2]]);

            fields.push(data_field(
                "SKSV",
                15,
                Some("7"),
                "1",
                "Sense key specific data is valid",
            ));
            fields.push(data_field(
                "C/D",
                15,
                Some("6"),
                format!("{}", cd),
                if cd != 0 {
                    "Error in Command (CDB)"
                } else {
                    "Error in Data parameters"
                },
            ));
            fields.push(data_field(
                "BPV",
                15,
                Some("3"),
                format!("{}", bpv),
                if bpv != 0 {
                    "Bit pointer is valid"
                } else {
                    "Bit pointer is not valid"
                },
            ));
            if bpv != 0 {
                fields.push(data_field(
                    "BIT POINTER",
                    15,
                    Some("2:0"),
                    format!("{}", bit_pointer),
                    format!("Bit {} of the byte in error", bit_pointer),
                ));
            }
            fields.push(data_field(
                "FIELD POINTER",
                16,
                None,
                format!("{:04X}", field_pointer),
                format!(
                    "Byte {} of {} in error",
                    field_pointer,
                    if cd != 0 { "CDB" } else { "data" }
                ),
            ));
        }
        // RECOVERED ERROR, MEDIUM ERROR, HARDWARE ERROR: actual retry count
        0x01 | 0x03 | 0x04 => {
            let retry_count = u16::from_be_bytes([sks[1], sks[2]]);
            fields.push(data_field(
                "SKSV",
                15,
                Some("7"),
                "1",
                "Sense key specific data is valid",
            ));
            fields.push(data_field(
                "ACTUAL RETRY COUNT",
                16,
                None,
                format!("{:04X}", retry_count),
                format!("{} retries", retry_count),
            ));
        }
        // NOT READY: progress indication
        0x02 => {
            let progress = u16::from_be_bytes([sks[1], sks[2]]);
            let pct = (progress as f64 / 65536.0) * 100.0;
            fields.push(data_field(
                "SKSV",
                15,
                Some("7"),
                "1",
                "Sense key specific data is valid",
            ));
            fields.push(data_field(
                "PROGRESS INDICATION",
                16,
                None,
                format!("{:04X}", progress),
                format!("{:.1}% complete", pct),
            ));
        }
        _ => {
            fields.push(data_field(
                "SENSE KEY SPECIFIC",
                15,
                None,
                format!("{:02X} {:02X} {:02X}", sks[0], sks[1], sks[2]),
                format!("SKSV=1 (sense key 0x{:X})", sense_key),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_sense_parsing() {
        let sense = [
            0xF0, // VALID=1, response code 70h
            0x00, // obsolete
            0xE5, // FILEMARK=1, EOM=1, ILI=1, sense key=5
            0x00, 0x00, 0x00, 0x10, // INFORMATION = 0x10
            0x0A, // additional sense length
            0x00, 0x00, 0x00, 0x00, // command specific
            0x24, 0x00, // ASC=24h ASCQ=00h
            0x00, // FRU
            0xC8, 0x00, 0x03, // SKS: SKSV=1 C/D=1 BPV=1 bit=0 field_ptr=3
        ];
        let sb = decode_sense(&sense);
        assert_eq!(sb.sense_key, 0x05);
        let fields = sb.fields.unwrap();
        let valid = fields.iter().find(|f| f.name == "VALID").unwrap();
        assert_eq!(valid.hex_value, "1");
        let fm = fields.iter().find(|f| f.name == "FILEMARK").unwrap();
        assert_eq!(fm.hex_value, "1");
        let fp = fields.iter().find(|f| f.name == "FIELD POINTER").unwrap();
        assert!(fp.decoded.contains("3"));
    }

    #[test]
    fn progress_indication_sense() {
        let sense = [
            0x70, 0x00, 0x02, // NOT READY
            0x00, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, // becoming ready
            0x00, 0x80, 0x80, 0x00, // SKSV=1, progress=0x8000 ~50%
        ];
        let sb = decode_sense(&sense);
        let fields = sb.fields.unwrap();
        let progress = fields
            .iter()
            .find(|f| f.name == "PROGRESS INDICATION")
            .unwrap();
        assert!(progress.decoded.contains("50"));
    }

    #[test]
    fn minimal_sense() {
        let sense = [0x70, 0x00, 0x00];
        let sb = decode_sense(&sense);
        assert_eq!(sb.sense_key, 0x00);
        assert!(sb.fields.unwrap().len() >= 5);
    }
}
