//! REPORT DENSITY SUPPORT command handler — stub.

use crate::sense;
use crate::ScsiResult;

/// Handle REPORT DENSITY SUPPORT (44h) — stub returning LTO-9.
pub fn handle_report_density_support(cdb: &[u8]) -> ScsiResult {
    let _media = cdb[1] & 0x01 != 0;
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    // Minimal response with one density descriptor (LTO-9)
    // Header: 2 bytes available length + 2 reserved
    let mut data = vec![0u8; 4];

    // Density descriptor (52 bytes)
    let mut desc = vec![0u8; 52];
    desc[0] = 0x60; // Primary density code (LTO-9)
    desc[1] = 0x60; // Secondary density code
    // Byte 2: DLV=0, DEFLT=1 (bit 5)
    desc[2] = 0x20;
    // Bytes 4-7: bits per mm (unused for us)
    // Bytes 8-9: media width (12700 = 12.65mm in tenths of mm)
    desc[8] = 0x31;
    desc[9] = 0x9C;
    // Bytes 10-11: tracks
    desc[10] = 0x00;
    desc[11] = 0x20; // 32 tracks
    // Bytes 12-15: capacity (MB) — 18000000 MB = 0x01312D00
    desc[12] = 0x01;
    desc[13] = 0x13;
    desc[14] = 0x2D;
    desc[15] = 0x00;
    // Bytes 16-23: Assigning organization "LTO-CVE " (8 bytes)
    desc[16..24].copy_from_slice(b"LTO-CVE ");
    // Bytes 24-31: Density name "LTO-9   " (8 bytes)
    desc[24..32].copy_from_slice(b"LTO-9   ");
    // Bytes 32-51: Description "Ultrium 9       " (20 bytes)
    desc[32..52].copy_from_slice(b"Ultrium 9           ");

    data.extend(&desc);

    // Fix header: available length
    let avail_len = (data.len() - 2) as u16;
    data[0] = ((avail_len >> 8) & 0xFF) as u8;
    data[1] = (avail_len & 0xFF) as u8;

    data.truncate(alloc_len);
    sense::good_with_data(data)
}
