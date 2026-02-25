//! Inquiry response decoders: standard inquiry + VPD pages.

use crate::cdb_decode::names::*;
use crate::cdb_decode::{data_field, data_field_parent, hex_string, DataField};

pub fn decode_standard_inquiry(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.is_empty() {
        return fields;
    }

    // Byte 0: Peripheral Qualifier (7:5) + Peripheral Device Type (4:0)
    let pq = (data[0] >> 5) & 0x07;
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Qualifier", 0, Some("7:5"), format!("{:X}", pq), peripheral_qualifier_name(pq)));
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));

    if data.len() < 2 { return fields; }

    // Byte 1: RMB (7)
    let rmb = (data[1] >> 7) & 1;
    fields.push(data_field("RMB", 1, Some("7"), format!("{}", rmb), if rmb != 0 { "Removable" } else { "Non-removable" }));

    if data.len() < 3 { return fields; }

    // Byte 2: VERSION
    fields.push(data_field("VERSION", 2, None, format!("{:02X}", data[2]), version_name(data[2])));

    if data.len() < 4 { return fields; }

    // Byte 3: NORMACA (5), HISUP (4), Response Data Format (3:0)
    let normaca = (data[3] >> 5) & 1;
    let hisup = (data[3] >> 4) & 1;
    let rdf = data[3] & 0x0F;
    fields.push(data_field("NORMACA", 3, Some("5"), format!("{}", normaca), if normaca != 0 { "ACA supported" } else { "ACA not supported" }));
    fields.push(data_field("HISUP", 3, Some("4"), format!("{}", hisup), if hisup != 0 { "Hierarchical LUN support" } else { "No hierarchical LUN" }));
    fields.push(data_field("Response Data Format", 3, Some("3:0"), format!("{}", rdf), if rdf == 2 { "SPC-2/3/4/5 compliant".into() } else { format!("Format {}", rdf) }));

    if data.len() < 5 { return fields; }

    // Byte 4: ADDITIONAL LENGTH
    fields.push(data_field("Additional Length", 4, None, format!("{:02X}", data[4]), format!("{} bytes follow", data[4])));

    if data.len() < 6 { return fields; }

    // Byte 5: SCCS, ACC, TPGS, 3PC, PROTECT
    let sccs = (data[5] >> 7) & 1;
    let acc = (data[5] >> 6) & 1;
    let tpgs = (data[5] >> 4) & 0x03;
    let _3pc = (data[5] >> 3) & 1;
    let protect = data[5] & 1;
    fields.push(data_field("SCCS", 5, Some("7"), format!("{}", sccs), if sccs != 0 { "Embedded storage array controller" } else { "No embedded controller" }));
    fields.push(data_field("ACC", 5, Some("6"), format!("{}", acc), if acc != 0 { "Access controls coordinator" } else { "No ACC" }));
    fields.push(data_field("TPGS", 5, Some("5:4"), format!("{}", tpgs), match tpgs { 0 => "Not supported", 1 => "Implicit only", 2 => "Explicit only", 3 => "Implicit and explicit", _ => "Unknown" }));
    fields.push(data_field("3PC", 5, Some("3"), format!("{}", _3pc), if _3pc != 0 { "Third-party copy supported" } else { "Not supported" }));
    fields.push(data_field("PROTECT", 5, Some("0"), format!("{}", protect), if protect != 0 { "Protection supported" } else { "No protection" }));

    if data.len() < 7 { return fields; }

    // Byte 6: obsolete, ENCSERV, VS, MULTIP
    let encserv = (data[6] >> 6) & 1;
    let multip = (data[6] >> 4) & 1;
    fields.push(data_field("ENCSERV", 6, Some("6"), format!("{}", encserv), if encserv != 0 { "Enclosure services" } else { "No enclosure" }));
    fields.push(data_field("MULTIP", 6, Some("4"), format!("{}", multip), if multip != 0 { "Multi-port device" } else { "Single port" }));

    if data.len() < 8 { return fields; }

    // Byte 7: CMDQUE, VS
    let cmdque = (data[7] >> 1) & 1;
    fields.push(data_field("CMDQUE", 7, Some("1"), format!("{}", cmdque), if cmdque != 0 { "Command queuing supported" } else { "No command queuing" }));

    // Bytes 8-15: T10 VENDOR IDENTIFICATION
    if data.len() >= 16 {
        let vendor = ascii_field(&data[8..16]);
        fields.push(data_field("T10 Vendor Identification", 8, None, hex_string(&data[8..16]), vendor));
    }

    // Bytes 16-31: PRODUCT IDENTIFICATION
    if data.len() >= 32 {
        let product = ascii_field(&data[16..32]);
        fields.push(data_field("Product Identification", 16, None, hex_string(&data[16..32]), product));
    }

    // Bytes 32-35: PRODUCT REVISION LEVEL
    if data.len() >= 36 {
        let rev = ascii_field(&data[32..36]);
        fields.push(data_field("Product Revision Level", 32, None, hex_string(&data[32..36]), rev));
    }

    // Bytes 36-55: Vendor specific (if present)
    if data.len() > 36 {
        let vs_end = data.len().min(56);
        if vs_end > 36 {
            fields.push(data_field("Vendor Specific", 36, None, hex_string(&data[36..vs_end]), format!("{} bytes", vs_end - 36)));
        }
    }

    // Bytes 56-57: Reserved
    // Bytes 58-73: Version descriptors (pairs of 2 bytes each, up to 8)
    if data.len() >= 60 {
        let num_descs = ((data.len().min(74)) - 58) / 2;
        for i in 0..num_descs {
            let off = 58 + i * 2;
            if off + 1 < data.len() {
                let ver = u16::from_be_bytes([data[off], data[off + 1]]);
                if ver != 0 {
                    fields.push(data_field(
                        format!("Version Descriptor {}", i + 1),
                        off, None,
                        format!("{:04X}", ver),
                        version_descriptor_name(ver),
                    ));
                }
            }
        }
    }

    fields
}

pub fn decode_vpd_page(page: u8, data: &[u8]) -> Vec<DataField> {
    match page {
        0x00 => decode_vpd_supported_pages(data),
        0x80 => decode_vpd_serial(data),
        0x83 => decode_vpd_device_id(data),
        0x86 => decode_vpd_extended_inquiry(data),
        0xB0 => decode_vpd_seq_access_caps(data),
        0xB1 => decode_vpd_mfg_serial(data),
        _ => decode_vpd_generic(page, data),
    }
}

fn decode_vpd_supported_pages(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Supported VPD Pages"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    let end = (4 + page_len).min(data.len());
    for i in 4..end {
        fields.push(data_field(
            format!("Supported Page"),
            i, None,
            format!("{:02X}", data[i]),
            vpd_page_name(data[i]),
        ));
    }
    fields
}

fn decode_vpd_serial(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Unit Serial Number"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    let end = (4 + page_len).min(data.len());
    if end > 4 {
        let serial = ascii_field(&data[4..end]);
        fields.push(data_field("Serial Number", 4, None, hex_string(&data[4..end]), serial));
    }
    fields
}

fn decode_vpd_device_id(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Device Identification"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    let mut off = 4;
    let end = (4 + page_len).min(data.len());
    let mut desc_num = 1;
    while off + 4 <= end {
        let protocol = (data[off] >> 4) & 0x0F;
        let code_set = data[off] & 0x0F;
        let piv = (data[off + 1] >> 7) & 1;
        let assoc = (data[off + 1] >> 4) & 0x03;
        let desig_type = data[off + 1] & 0x0F;
        let desig_len = data[off + 3] as usize;

        let id_end = (off + 4 + desig_len).min(end);
        let id_data = if id_end > off + 4 { &data[off + 4..id_end] } else { &[] as &[u8] };

        let id_decoded = if code_set == 0x02 {
            ascii_field(id_data)
        } else {
            hex_string(id_data)
        };

        let children = vec![
            data_field("Protocol Identifier", off, Some("7:4"), format!("{:X}", protocol), protocol_identifier_name(protocol)),
            data_field("Code Set", off, Some("3:0"), format!("{:X}", code_set), code_set_name(code_set)),
            data_field("PIV", off + 1, Some("7"), format!("{}", piv), if piv != 0 { "Protocol identifier valid" } else { "Not valid" }),
            data_field("Association", off + 1, Some("5:4"), format!("{:X}", assoc), association_name(assoc)),
            data_field("Designator Type", off + 1, Some("3:0"), format!("{:X}", desig_type), designator_type_name(desig_type)),
            data_field("Designator Length", off + 3, None, format!("{:02X}", desig_len), format!("{} bytes", desig_len)),
            data_field("Identifier", off + 4, None, hex_string(id_data), id_decoded),
        ];

        fields.push(data_field_parent(
            format!("Designation Descriptor {}", desc_num),
            off, None,
            hex_string(&data[off..id_end.min(data.len())]),
            designator_type_name(desig_type),
            children,
        ));

        off = off + 4 + desig_len;
        desc_num += 1;
    }
    fields
}

fn decode_vpd_extended_inquiry(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Extended INQUIRY Data"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    if data.len() >= 5 {
        let spt = (data[4] >> 3) & 0x07;
        let grd_chk = (data[4] >> 2) & 1;
        let app_chk = (data[4] >> 1) & 1;
        let ref_chk = data[4] & 1;
        fields.push(data_field("SPT", 4, Some("5:3"), format!("{}", spt), format!("Protection types supported: {}", spt)));
        fields.push(data_field("GRD_CHK", 4, Some("2"), format!("{}", grd_chk), if grd_chk != 0 { "Guard checking" } else { "No guard check" }));
        fields.push(data_field("APP_CHK", 4, Some("1"), format!("{}", app_chk), if app_chk != 0 { "Application tag checking" } else { "No app check" }));
        fields.push(data_field("REF_CHK", 4, Some("0"), format!("{}", ref_chk), if ref_chk != 0 { "Reference tag checking" } else { "No ref check" }));
    }
    if data.len() >= 6 {
        let simpsup = (data[5] >> 4) & 1;
        let ordsup = (data[5] >> 3) & 1;
        let headsup = (data[5] >> 2) & 1;
        fields.push(data_field("SIMPSUP", 5, Some("4"), format!("{}", simpsup), if simpsup != 0 { "Simple task support" } else { "No" }));
        fields.push(data_field("ORDSUP", 5, Some("3"), format!("{}", ordsup), if ordsup != 0 { "Ordered task support" } else { "No" }));
        fields.push(data_field("HEADSUP", 5, Some("2"), format!("{}", headsup), if headsup != 0 { "Head of queue support" } else { "No" }));
    }
    fields
}

fn decode_vpd_seq_access_caps(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Sequential-Access Device Capabilities"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    if data.len() >= 5 {
        let worm = (data[4] >> 0) & 1;
        fields.push(data_field("WORM", 4, Some("0"), format!("{}", worm), if worm != 0 { "WORM support" } else { "No WORM" }));
    }
    fields
}

fn decode_vpd_mfg_serial(data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", data[1]), "Manufacturer-assigned Serial Number"));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));

    let end = (4 + page_len).min(data.len());
    if end > 4 {
        let serial = ascii_field(&data[4..end]);
        fields.push(data_field("Manufacturer Serial Number", 4, None, hex_string(&data[4..end]), serial));
    }
    fields
}

fn decode_vpd_generic(page: u8, data: &[u8]) -> Vec<DataField> {
    let mut fields = Vec::new();
    if data.len() < 4 { return fields; }
    let pdt = data[0] & 0x1F;
    fields.push(data_field("Peripheral Device Type", 0, Some("4:0"), format!("{:02X}", pdt), peripheral_device_type_name(pdt)));
    fields.push(data_field("Page Code", 1, None, format!("{:02X}", page), vpd_page_name(page)));
    let page_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    fields.push(data_field("Page Length", 2, None, format!("{:04X}", page_len), format!("{} bytes", page_len)));
    if data.len() > 4 {
        fields.push(data_field("Page Data", 4, None, hex_string(&data[4..]), format!("{} bytes", data.len() - 4)));
    }
    fields
}

fn ascii_field(data: &[u8]) -> String {
    let s: String = data.iter().map(|&b| if b >= 0x20 && b <= 0x7E { b as char } else { '.' }).collect();
    s.trim().to_string()
}

fn version_descriptor_name(ver: u16) -> String {
    match ver {
        0x0000 => "No standard".into(),
        0x0020 => "SAM (no version)".into(),
        0x003C => "SAM-2 (no version)".into(),
        0x0060 => "SAM-3 (no version)".into(),
        0x0077 => "SAM-3 T10/1561-D rev 14".into(),
        0x0080 => "SAM-4 (no version)".into(),
        0x0120 => "SPC (no version)".into(),
        0x013C => "SPC T10/0995-D rev 11a".into(),
        0x0260 => "SPC-3 (no version)".into(),
        0x0300 => "SPC-4 (no version)".into(),
        0x0460 => "SPC-5 (no version)".into(),
        0x0320 => "SBC-2 (no version)".into(),
        0x0360 => "SBC-3 (no version)".into(),
        0x0500 => "SMC (no version)".into(),
        0x055C => "SMC-2 (no version)".into(),
        0x0560 => "SMC-3 (no version)".into(),
        0x0180 => "SCC (no version)".into(),
        0x0200 => "SES (no version)".into(),
        0x0380 => "SSC-2 (no version)".into(),
        0x03A0 => "SSC-2 T10/1434-D rev 9".into(),
        0x03C0 => "SSC-3 (no version)".into(),
        0x0400 => "SSC-4 (no version)".into(),
        0x0820 => "SSA-S3P (no version)".into(),
        0x0BE0 => "SAS-1 T10/1562-D rev 10".into(),
        0x0C00 => "SAS-1.1 (no version)".into(),
        0x0D20 => "SAS-2 (no version)".into(),
        0x0D40 => "SAS-2 T10/1760-D rev 14".into(),
        0x0D60 => "SAS-2.1 (no version)".into(),
        0x0E00 => "SAS-3 (no version)".into(),
        0x0F00 => "SAS-4 (no version)".into(),
        0x0A00 => "SPI-2 (no version)".into(),
        0x0AA0 => "SPI-4 (no version)".into(),
        0x0AB0..=0x0ABF => "SPI-4".into(),
        0x0AC0 => "SPI-5 (no version)".into(),
        _ => format!("Version 0x{:04X}", ver),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_inquiry_response() {
        // Typical tape drive standard inquiry response (36 bytes minimum)
        let mut data = vec![0u8; 96];
        data[0] = 0x01; // Sequential-access
        data[1] = 0x80; // RMB=1
        data[2] = 0x06; // SPC-4
        data[3] = 0x02; // Response format 2
        data[4] = 91;   // Additional length
        // Vendor (bytes 8-15): "QUANTUM "
        data[8..16].copy_from_slice(b"QUANTUM ");
        // Product (bytes 16-31): "ULTRIUM-HH8     "
        data[16..32].copy_from_slice(b"ULTRIUM-HH8     ");
        // Revision (bytes 32-35): "0001"
        data[32..36].copy_from_slice(b"0001");

        let fields = decode_standard_inquiry(&data);
        assert!(fields.iter().any(|f| f.name == "Peripheral Device Type" && f.decoded.contains("Sequential")));
        assert!(fields.iter().any(|f| f.name == "RMB" && f.decoded.contains("Removable")));
        assert!(fields.iter().any(|f| f.name == "T10 Vendor Identification" && f.decoded.contains("QUANTUM")));
        assert!(fields.iter().any(|f| f.name == "Product Identification" && f.decoded.contains("ULTRIUM-HH8")));
    }

    #[test]
    fn vpd_serial_number() {
        let data = [
            0x01, 0x80, 0x00, 0x0A,
            b'H', b'U', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8',
        ];
        let fields = decode_vpd_page(0x80, &data);
        assert!(fields.iter().any(|f| f.name == "Serial Number" && f.decoded.contains("HU12345678")));
    }

    #[test]
    fn vpd_device_id() {
        let mut data = vec![0x01, 0x83, 0x00, 0x10]; // page len 16
        // One descriptor: ASCII, NAA type 3, LU association
        data.push(0x02); // code_set=ASCII
        data.push(0x03); // assoc=0 (LU), type=3 (NAA)
        data.push(0x00); // reserved
        data.push(0x08); // desig_len=8
        data.extend_from_slice(b"TEST1234");
        // Pad to page_len
        while data.len() < 4 + 16 { data.push(0); }

        let fields = decode_vpd_page(0x83, &data);
        assert!(fields.iter().any(|f| f.name.contains("Designation Descriptor")));
    }
}
