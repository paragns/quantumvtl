//! Maintenance commands: REPORT SUPPORTED OPERATION CODES, TMF, TIMESTAMP, READ BUFFER.

use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;

/// Handle READ BUFFER (3Ch) — returns drive buffer descriptor or data.
/// StorNext uses this to verify drive buffer capability before writing.
pub fn handle_read_buffer(cdb: &[u8]) -> ScsiResult {
    let mode = cdb[1] & 0x1F;
    let alloc_len = ((cdb[6] as usize) << 16) | ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    // Buffer capacity: 256 MB capped to 24-bit max (0xFFFFFF = ~16 MB)
    // Report 16 MB — sufficient for StorNext compatibility check
    const BUF_CAP: u32 = 0x00FF_FFFF; // 16,777,215 bytes

    let mut data = match mode {
        0x03 => {
            // Descriptor mode: 4 bytes — offset boundary + 24-bit capacity
            vec![
                0x00,                          // offset boundary (no alignment required)
                ((BUF_CAP >> 16) & 0xFF) as u8,
                ((BUF_CAP >> 8) & 0xFF) as u8,
                (BUF_CAP & 0xFF) as u8,
            ]
        }
        0x00 => {
            // Combined header + data: 4-byte header followed by zeroed buffer data
            let mut d = vec![
                0x00,
                ((BUF_CAP >> 16) & 0xFF) as u8,
                ((BUF_CAP >> 8) & 0xFF) as u8,
                (BUF_CAP & 0xFF) as u8,
            ];
            let data_len = alloc_len.saturating_sub(4).min(BUF_CAP as usize);
            d.extend(vec![0u8; data_len]);
            d
        }
        0x02 => {
            // Data only: return zeroed buffer contents
            vec![0u8; alloc_len.min(BUF_CAP as usize)]
        }
        0x0B => {
            // Echo buffer descriptor: 4 bytes
            vec![0x00, 0x00, 0x01, 0x00] // 256-byte echo buffer
        }
        _ => {
            // Unsupported mode — return empty response rather than failing
            vec![0u8; 0]
        }
    };

    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle MAINTENANCE IN (A3h) — service action dispatch.
pub fn handle_maintenance_in(cdb: &[u8]) -> ScsiResult {
    let service_action = cdb[1] & 0x1F;
    let alloc_len = ((cdb[6] as usize) << 24)
        | ((cdb[7] as usize) << 16)
        | ((cdb[8] as usize) << 8)
        | (cdb[9] as usize);

    match service_action {
        0x0C => handle_report_supported_opcodes(cdb, alloc_len),
        0x0D => handle_report_supported_tmf(alloc_len),
        0x0F => handle_report_timestamp(alloc_len),
        _ => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}

/// Handle MAINTENANCE OUT (A4h) — service action dispatch.
pub fn handle_maintenance_out(cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
    let service_action = cdb[1] & 0x1F;

    match service_action {
        0x02 => {
            // SET TIMESTAMP — stub, accept and ignore
            sense::good()
        }
        _ => SenseBuilder::invalid_field_in_cdb().to_check_condition(),
    }
}

/// REPORT SUPPORTED OPERATION CODES (A3h[0Ch]) — stub returning "all opcodes" format.
fn handle_report_supported_opcodes(_cdb: &[u8], alloc_len: usize) -> ScsiResult {
    // Reporting options in byte 2 bits 2-0:
    // For now, return a minimal response
    let mut data = vec![0u8; 4]; // 4-byte header with command data length = 0
    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// REPORT SUPPORTED TASK MANAGEMENT FUNCTIONS (A3h[0Dh]) — stub.
fn handle_report_supported_tmf(alloc_len: usize) -> ScsiResult {
    // Minimal: support ABORT TASK, LUN RESET
    let mut data = vec![0u8; 4];
    data[0] = 0x80 | 0x08; // ABORT TASK + LOGICAL UNIT RESET
    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// REPORT TIMESTAMP (A3h[0Fh]) — stub.
fn handle_report_timestamp(alloc_len: usize) -> ScsiResult {
    // Return current time as milliseconds since midnight Jan 1, 1970
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut data = vec![0u8; 12];
    // Bytes 0-1: parameter data length (10)
    data[0] = 0x00;
    data[1] = 0x0A;
    // Byte 2: origin (00 = initialized to zero at power on)
    data[2] = 0x00;
    // Bytes 4-9: timestamp (48-bit milliseconds)
    data[4] = ((now_ms >> 40) & 0xFF) as u8;
    data[5] = ((now_ms >> 32) & 0xFF) as u8;
    data[6] = ((now_ms >> 24) & 0xFF) as u8;
    data[7] = ((now_ms >> 16) & 0xFF) as u8;
    data[8] = ((now_ms >> 8) & 0xFF) as u8;
    data[9] = (now_ms & 0xFF) as u8;

    data.truncate(alloc_len);
    sense::good_with_data(data)
}
