//! Log page framework for the media changer.
//!
//! The Quantum Scalar spec defines these log pages:
//! - 00h: Supported Log Pages
//! - 0Dh: Temperature
//! - 12h: TapeAlert Response (bitmap)
//! - 2Eh: TapeAlert (individual flags)
//! - 30h: Humidity

use std::sync::Mutex;

/// Registry of log pages.
pub struct LogPageRegistry {
    /// Page codes that are supported.
    supported: Vec<u8>,
    /// Mutable state for updateable values.
    state: Mutex<LogState>,
}

/// Mutable log state (temperature, humidity, alerts).
struct LogState {
    temperature_c: u8,
    humidity_pct: u8,
    tape_alerts: [bool; 64],
}

impl LogPageRegistry {
    pub fn new() -> Self {
        Self {
            supported: vec![0x00, 0x0D, 0x12, 0x2E, 0x30],
            state: Mutex::new(LogState {
                temperature_c: 22,
                humidity_pct: 45,
                tape_alerts: [false; 64],
            }),
        }
    }

    /// Get the list of supported page codes.
    pub fn supported_pages(&self) -> Vec<u8> {
        self.supported.clone()
    }

    /// Get a log page's full response data (header + parameters).
    pub fn get_page(&self, page_code: u8) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();

        match page_code {
            0x0D => Some(build_temperature_page(state.temperature_c)),
            0x12 => Some(build_tape_alert_response_page(&state.tape_alerts)),
            0x2E => Some(build_tape_alert_page(&state.tape_alerts)),
            0x30 => Some(build_humidity_page(state.humidity_pct)),
            _ => None,
        }
    }

    /// Update simulated temperature.
    pub fn set_temperature(&self, temp_c: u8) {
        self.state.lock().unwrap().temperature_c = temp_c;
    }

    /// Update simulated humidity.
    pub fn set_humidity(&self, humidity_pct: u8) {
        self.state.lock().unwrap().humidity_pct = humidity_pct;
    }

    /// Set a TapeAlert flag (1-based index, 1-64).
    pub fn set_alert(&self, flag: u8, active: bool) {
        if flag >= 1 && flag <= 64 {
            self.state.lock().unwrap().tape_alerts[(flag - 1) as usize] = active;
        }
    }

    /// Clear all TapeAlert flags.
    pub fn clear_alerts(&self) {
        self.state.lock().unwrap().tape_alerts = [false; 64];
    }
}

/// Build the default log page registry.
pub fn default_registry() -> LogPageRegistry {
    LogPageRegistry::new()
}

// ── Page builders ──────────────────────────────────────────────────────────

/// Build Temperature log page (0Dh).
fn build_temperature_page(temp_c: u8) -> Vec<u8> {
    let mut data = Vec::with_capacity(8);
    // Page header
    data.push(0x0D); // Page code
    data.push(0x00); // Subpage code
    data.push(0x00); // Page length MSB
    data.push(0x06); // Page length LSB (6 bytes of parameters)

    // Parameter: code 0000h, 2-byte value
    data.push(0x00); // Parameter code MSB
    data.push(0x00); // Parameter code LSB
    data.push(0x03); // Control: binary, list format
    data.push(0x02); // Parameter length
    data.push(0x00); // Reserved
    data.push(temp_c); // Temperature in °C
    data
}

/// Build Humidity log page (30h).
fn build_humidity_page(humidity_pct: u8) -> Vec<u8> {
    let mut data = Vec::with_capacity(8);
    data.push(0x30);
    data.push(0x00);
    data.push(0x00);
    data.push(0x06);

    data.push(0x00);
    data.push(0x00);
    data.push(0x03);
    data.push(0x02);
    data.push(0x00);
    data.push(humidity_pct);
    data
}

/// Build TapeAlert log page (2Eh) — 64 individual parameters.
fn build_tape_alert_page(alerts: &[bool; 64]) -> Vec<u8> {
    // Header + 64 parameters × (4 header + 1 value) = 4 + 320 = 324 bytes
    let param_count = 64;
    let params_len = param_count * 5;
    let mut data = Vec::with_capacity(4 + params_len);

    data.push(0x2E);
    data.push(0x00);
    data.push(((params_len >> 8) & 0xFF) as u8);
    data.push((params_len & 0xFF) as u8);

    for i in 0..64u16 {
        let code = i + 1; // Flags are 1-based (0001h-0040h)
        data.push((code >> 8) as u8);
        data.push(code as u8);
        data.push(0x03); // Control: binary
        data.push(0x01); // Parameter length
        data.push(if alerts[i as usize] { 0x01 } else { 0x00 });
    }
    data
}

/// Build TapeAlert Response log page (12h) — bitmap format.
fn build_tape_alert_response_page(alerts: &[bool; 64]) -> Vec<u8> {
    let mut data = Vec::with_capacity(16);
    data.push(0x12);
    data.push(0x00);
    data.push(0x00);
    data.push(0x0C); // Page length (12 bytes: parameter header + 8-byte bitmap)

    // Single parameter: code 0000h, 8-byte value
    data.push(0x00);
    data.push(0x00);
    data.push(0x03); // Control: binary
    data.push(0x08); // Parameter length

    // Build 8-byte bitmap (flag 1 = byte 0 bit 7, flag 8 = byte 0 bit 0, etc.)
    let mut bitmap = [0u8; 8];
    for i in 0..64 {
        if alerts[i] {
            let byte_idx = i / 8;
            let bit_idx = 7 - (i % 8);
            bitmap[byte_idx] |= 1 << bit_idx;
        }
    }
    data.extend_from_slice(&bitmap);
    data
}
