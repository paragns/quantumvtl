//! VPD (Vital Product Data) page implementations.
//!
//! Quantum Scalar spec requires: 00h, 80h, 83h, 85h, C8h.

/// Dispatch to the appropriate VPD page builder.
pub fn handle_vpd_page(
    page_code: u8,
    serial: &str,
    vendor: &str,
    product: &str,
) -> Option<Vec<u8>> {
    match page_code {
        0x00 => Some(build_page_00()),
        0x80 => Some(build_page_80(serial)),
        0x83 => Some(build_page_83(vendor, product, serial)),
        0x85 => Some(build_page_85()),
        0xC8 => Some(build_page_c8()),
        _ => None,
    }
}

/// VPD page 00h: Supported VPD Pages.
fn build_page_00() -> Vec<u8> {
    vec![
        0x08, // Peripheral qualifier (0) | Device type (08h = Medium Changer)
        0x00, // Page code
        0x00, // Reserved
        0x05, // Page length
        0x00, 0x80, 0x83, 0x85, 0xC8, // Supported pages
    ]
}

/// VPD page 80h: Unit Serial Number.
fn build_page_80(serial: &str) -> Vec<u8> {
    let serial_bytes = serial.as_bytes();
    // Pad to 24 bytes per spec
    let mut padded = vec![0x20u8; 24]; // space-padded
    let copy_len = serial_bytes.len().min(24);
    padded[..copy_len].copy_from_slice(&serial_bytes[..copy_len]);

    let mut data = vec![
        0x08, // Device type
        0x80, // Page code
        0x00, // Reserved
        padded.len() as u8, // Page length
    ];
    data.extend_from_slice(&padded);
    data
}

/// VPD page 83h: Device Identification.
fn build_page_83(vendor: &str, _product: &str, serial: &str) -> Vec<u8> {
    let mut data = vec![0x08, 0x83, 0x00, 0x00]; // Header (length filled later)

    // Descriptor 1: T10 vendor ID based (Association=0, Type=1)
    let t10_id = format!("{:<8}{:<24}", vendor.trim(), serial);
    let t10_bytes = t10_id.as_bytes();
    data.push(0x02); // Code set = ASCII
    data.push(0x01); // Association=0 (device), Type=1 (T10 vendor ID)
    data.push(0x00); // Reserved
    data.push(t10_bytes.len() as u8);
    data.extend_from_slice(t10_bytes);

    // Descriptor 2: NAA (Network Address Authority) for WWNN
    // Build a plausible NAA-5 identifier from serial hash
    let hash = simple_hash(serial);
    let mut naa = [0u8; 8];
    naa[0] = 0x50; // NAA=5, top nibble of company ID
    // Company ID: 00308C (Quantum/ADIC) → spread across bytes 0-2
    naa[0] |= 0x00; // NAA=5, company MSN=0
    naa[1] = 0x03;
    naa[2] = 0x08;
    naa[3] = 0xC0 | ((hash >> 28) & 0x0F) as u8;
    naa[4] = ((hash >> 20) & 0xFF) as u8;
    naa[5] = ((hash >> 12) & 0xFF) as u8;
    naa[6] = ((hash >> 4) & 0xFF) as u8;
    naa[7] = ((hash << 4) & 0xF0) as u8;

    data.push(0x01); // Code set = binary
    data.push(0x63); // PIV=1, Association=0, Type=3 (NAA)
    data.push(0x00);
    data.push(0x08); // 8 bytes
    data.extend_from_slice(&naa);

    // Fill page length
    let page_len = (data.len() - 4) as u8;
    data[3] = page_len;
    data
}

/// VPD page 85h: Management Network Addresses.
fn build_page_85() -> Vec<u8> {
    // Return a minimal page with a single management URL placeholder.
    // The actual URL would be set at runtime based on config.
    let url = b"https://localhost:8443/";
    let descriptor_len = 4 + url.len();
    let page_len = descriptor_len;

    let mut data = vec![
        0x08, // Device type
        0x85, // Page code
        (page_len >> 8) as u8,
        page_len as u8,
    ];

    // Descriptor: Service Type 03h (Management/Status)
    data.push(0x00); // Association=0
    data.push(0x03); // Service Type (Management)
    data.push((url.len() >> 8) as u8);
    data.push(url.len() as u8);
    data.extend_from_slice(url);

    data
}

/// VPD page C8h: Vendor-Specific Device Capabilities.
fn build_page_c8() -> Vec<u8> {
    vec![
        0x08, // Device type
        0xC8, // Page code
        0x00, // Reserved
        0x04, // Page length
        0x00, // Byte 4: ADVFO=0, BASICFO=0
        0x00, // Reserved
        0x00, // Reserved
        0x00, // Reserved
    ]
}

/// Simple hash function for generating plausible NAA identifiers.
fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in s.as_bytes() {
        hash = hash.wrapping_mul(0x01000193);
        hash ^= b as u32;
    }
    hash
}
