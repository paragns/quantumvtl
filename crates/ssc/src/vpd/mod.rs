//! INQUIRY VPD (Vital Product Data) page dispatch and implementations.
//!
//! Each VPD page is a function that returns the page data. The dispatch
//! function routes based on page code. Pages are filled in by Track D.

use crate::sense;

/// Handle INQUIRY with EVPD=1 — dispatch to the appropriate VPD page.
pub fn handle_vpd_page(page_code: u8, alloc_len: usize, serial: &str) -> crate::ScsiResult {
    let data = match page_code {
        0x00 => page_00_supported(serial),
        0x03 => page_03_firmware(serial),
        0x80 => page_80_serial(serial),
        0x83 => page_83_identification(serial),
        0x86 => page_86_extended_inquiry(),
        0xB0 => page_b0_seq_access_capabilities(),
        0xB5 => page_b5_block_protection(),
        0xC0 => page_c0_component_revisions(),
        _ => {
            return sense::SenseBuilder::invalid_field_in_cdb().to_check_condition();
        }
    };

    let mut result = data;
    result.truncate(alloc_len);
    sense::good_with_data(result)
}

/// VPD 00h: Supported VPD Pages
fn page_00_supported(_serial: &str) -> Vec<u8> {
    let pages = [0x00, 0x03, 0x80, 0x83, 0x86, 0xB0, 0xB5, 0xC0];
    let mut data = vec![
        0x01,              // peripheral qualifier + device type (sequential access)
        0x00,              // page code
        0x00,              // reserved
        pages.len() as u8, // page length
    ];
    data.extend_from_slice(&pages);
    data
}

/// VPD 03h: Firmware Designation
fn page_03_firmware(_serial: &str) -> Vec<u8> {
    let mut data = vec![0x01, 0x03, 0x00, 0x00]; // header (length filled below)

    // Designation descriptor
    let load_id = b"0000"; // 4-byte LOAD ID
    let fw_rev = b"A1B0"; // 4-byte firmware revision (YMDV format)
    let ru_name = b"QUANTUMVTL      "; // 8-byte RU name (space-padded to 8)
                                       // PTF number
    let ptf = b"0000    "; // 8 bytes

    data.extend_from_slice(load_id);
    data.extend_from_slice(fw_rev);
    data.extend_from_slice(&ru_name[..8]);
    data.extend_from_slice(&ptf[..8]);

    // Pad to at least 33 bytes total
    while data.len() < 33 {
        data.push(0x00);
    }

    // Fix page length
    data[3] = (data.len() - 4) as u8;
    data
}

/// VPD 80h: Unit Serial Number
fn page_80_serial(serial: &str) -> Vec<u8> {
    // Serial is right-justified, 10 bytes, zero-padded on left
    let formatted = format!("{:0>10}", serial);
    let serial_bytes = formatted.as_bytes();

    let mut data = vec![
        0x01, // peripheral qualifier + device type
        0x80, // page code
        0x00, // reserved
        serial_bytes.len() as u8,
    ];
    data.extend_from_slice(serial_bytes);
    data
}

/// VPD 83h: Device Identification
fn page_83_identification(serial: &str) -> Vec<u8> {
    let mut data = vec![0x01, 0x83, 0x00, 0x00]; // header

    // Descriptor 1: T10 Vendor ID based
    let vendor = b"IBM     "; // 8 bytes
    let product = b"ULT3580-TD9     "; // 16 bytes — TODO: generation-specific
    let serial_bytes = format!("{:0>10}", serial);

    let mut desc1 = Vec::new();
    desc1.push(0x02); // code set = ASCII
    desc1.push(0x01); // PIV=0, association=0 (logical unit), identifier type=1 (T10 vendor ID)
    desc1.push(0x00); // reserved
    let id_data_start = desc1.len();
    desc1.push(0x00); // identifier length (filled below)
    desc1.extend_from_slice(vendor);
    desc1.extend_from_slice(product);
    desc1.extend_from_slice(serial_bytes.as_bytes());
    desc1[id_data_start] = (desc1.len() - id_data_start - 1) as u8;
    data.extend(&desc1);

    // Descriptor 2: NAA WWNN (fabricated)
    // NAA=5 (IEEE registered), OUI=fabricated
    let mut desc2 = Vec::new();
    desc2.push(0x01); // code set = binary
    desc2.push(0x03); // PIV=0, association=0, identifier type=3 (NAA)
    desc2.push(0x00); // reserved
    desc2.push(0x08); // identifier length = 8
                      // NAA 5 + OUI 00:11:22 + vendor-specific from serial hash
    let serial_hash = simple_hash(serial);
    desc2.extend_from_slice(&[
        0x50,
        0x01,
        0x12,
        0x20,
        ((serial_hash >> 24) & 0xFF) as u8,
        ((serial_hash >> 16) & 0xFF) as u8,
        ((serial_hash >> 8) & 0xFF) as u8,
        (serial_hash & 0xFF) as u8,
    ]);
    data.extend(&desc2);

    // Descriptor 3: Relative target port
    let mut desc3 = Vec::new();
    desc3.push(0x01); // binary
    desc3.push(0x14); // PIV=1, association=1 (target port), type=4 (relative target port)
    desc3.push(0x00);
    desc3.push(0x04); // length = 4
    desc3.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // relative target port = 1
    data.extend(&desc3);

    // Fix page length
    let page_len = (data.len() - 4) as u16;
    data[2] = ((page_len >> 8) & 0xFF) as u8;
    data[3] = (page_len & 0xFF) as u8;
    data
}

/// VPD 86h: Extended INQUIRY Data (64 bytes)
fn page_86_extended_inquiry() -> Vec<u8> {
    let mut data = vec![0x01, 0x86, 0x00, 60]; // header + 60 bytes
    data.resize(64, 0x00);
    // Byte 5: SIMPSUP=1 (simple queuing supported)
    data[5 + 4] = 0x01;
    data
}

/// VPD B0h: Sequential-Access Device Capabilities
fn page_b0_seq_access_capabilities() -> Vec<u8> {
    vec![
        0x01, 0xB0, // qualifier + page code
        0x00, 0x02, // page length = 2
        0x00, // WORM=0 (no WORM support yet)
        0x00, // reserved
    ]
}

/// VPD B5h: Logical Block Protection
fn page_b5_block_protection() -> Vec<u8> {
    let mut data = vec![0x01, 0xB5, 0x00, 0x00]; // header

    // Method 00h: LBP disabled (always available)
    data.extend_from_slice(&[
        0x00, // LBP method
        0x00, // method byte count (length of info following)
        0x00, // method-specific
        0x00,
    ]);

    // Fix page length
    data[3] = (data.len() - 4) as u8;
    data
}

/// VPD C0h: Drive Component Revision Levels
fn page_c0_component_revisions() -> Vec<u8> {
    let mut data = vec![0x01, 0xC0, 0x00, 0x00]; // header

    // Code name (12 bytes)
    data.extend_from_slice(b"QuantumVTL  ");
    // Time (7 bytes): HHMMSS + null
    data.extend_from_slice(b"120000\0");
    // Date (8 bytes): YYYYMMDD
    data.extend_from_slice(b"20260101");
    // Platform (12 bytes)
    data.extend_from_slice(b"iscsi_vtl   ");

    // Fix page length
    data[3] = (data.len() - 4) as u8;
    data
}

/// Simple deterministic hash of a string to produce a u32 for WWNN generation.
fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash
}
