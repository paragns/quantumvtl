//! Maintenance commands: REPORT SUPPORTED OPERATION CODES, TMF, TIMESTAMP — stubs.

use crate::sense::{self, SenseBuilder};
use crate::ScsiResult;

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
