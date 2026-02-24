//! Medium Auxiliary Memory (MAM) — cartridge memory attributes.
//!
//! MAM stores per-cartridge metadata that travels with the media.
//! Accessed via READ ATTRIBUTE (8Ch) and WRITE ATTRIBUTE (8Dh).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Well-known MAM attribute identifiers (from spec section 6.5).
pub mod attr {
    /// Remaining capacity in partition (bytes), per-partition.
    pub const REMAINING_CAPACITY: u16 = 0x0000;
    /// Maximum capacity in partition (bytes), per-partition.
    pub const MAXIMUM_CAPACITY: u16 = 0x0001;
    /// Tape alert flags.
    pub const TAPEALERT_FLAGS: u16 = 0x0002;
    /// Load count.
    pub const LOAD_COUNT: u16 = 0x0003;
    /// MAM space remaining (bytes).
    pub const MAM_SPACE_REMAINING: u16 = 0x0004;

    // Medium type attributes (0x0400-0x04FF)
    /// Medium manufacturer (ASCII).
    pub const MEDIUM_MANUFACTURER: u16 = 0x0400;
    /// Medium serial number (ASCII).
    pub const MEDIUM_SERIAL_NUMBER: u16 = 0x0401;
    /// Medium length (meters).
    pub const MEDIUM_LENGTH: u16 = 0x0402;
    /// Medium width (tenths of mm).
    pub const MEDIUM_WIDTH: u16 = 0x0403;
    /// Medium type.
    pub const MEDIUM_TYPE: u16 = 0x0408;
    /// Medium type information.
    pub const MEDIUM_TYPE_INFORMATION: u16 = 0x0409;

    // Host type attributes (0x0800-0x08FF)
    /// Application vendor (ASCII).
    pub const APPLICATION_VENDOR: u16 = 0x0800;
    /// Application name (ASCII).
    pub const APPLICATION_NAME: u16 = 0x0801;
    /// Application version (ASCII).
    pub const APPLICATION_VERSION: u16 = 0x0802;
    /// User medium text label (ASCII).
    pub const USER_MEDIUM_TEXT_LABEL: u16 = 0x0803;
    /// Barcode (ASCII).
    pub const BARCODE: u16 = 0x0806;

    // Device type attributes (0x0000-0x03FF written by device)
    /// Total bytes written in medium life (8 bytes).
    pub const TOTAL_MB_WRITTEN: u16 = 0x0220;
    /// Total bytes read in medium life (8 bytes).
    pub const TOTAL_MB_READ: u16 = 0x0222;

    // LTO-9 specific
    /// Medium optimization needed (1 byte: 0=no, 1=yes).
    pub const MEDIUM_OPTIMIZATION_NEEDED: u16 = 0x1010;
}

/// A single MAM attribute value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MamAttribute {
    /// Attribute identifier.
    pub id: u16,
    /// Whether this attribute is read-only.
    pub read_only: bool,
    /// Raw attribute value.
    pub value: Vec<u8>,
}

/// Medium Auxiliary Memory — per-cartridge attribute store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MamAttributes {
    attributes: BTreeMap<u16, MamAttribute>,
}

impl MamAttributes {
    /// Create a new MAM with default attributes for a tape cartridge.
    pub fn new_for_cartridge(barcode: &str, manufacturer: &str, serial: &str) -> Self {
        let mut mam = Self {
            attributes: BTreeMap::new(),
        };

        // Medium manufacturer
        mam.set_readonly(attr::MEDIUM_MANUFACTURER, pad_ascii(manufacturer, 8));
        // Medium serial
        mam.set_readonly(attr::MEDIUM_SERIAL_NUMBER, pad_ascii(serial, 32));
        // Barcode
        mam.set_writable(attr::BARCODE, pad_ascii(barcode, 32));
        // Load count
        mam.set_readonly(attr::LOAD_COUNT, vec![0, 0, 0, 0]);
        // Total MB written
        mam.set_readonly(attr::TOTAL_MB_WRITTEN, vec![0; 8]);
        // Total MB read
        mam.set_readonly(attr::TOTAL_MB_READ, vec![0; 8]);
        // Application vendor
        mam.set_writable(attr::APPLICATION_VENDOR, pad_ascii("", 8));
        // Application name
        mam.set_writable(attr::APPLICATION_NAME, pad_ascii("", 32));

        mam
    }

    /// Get an attribute by ID.
    pub fn get(&self, id: u16) -> Option<&MamAttribute> {
        self.attributes.get(&id)
    }

    /// Set a writable attribute.
    pub fn set(&mut self, id: u16, value: Vec<u8>) -> bool {
        if let Some(attr) = self.attributes.get_mut(&id) {
            if attr.read_only {
                return false;
            }
            attr.value = value;
            true
        } else {
            // New writable attribute
            self.attributes.insert(
                id,
                MamAttribute {
                    id,
                    read_only: false,
                    value,
                },
            );
            true
        }
    }

    /// Set a device-managed (read-only to host) attribute.
    pub fn set_device_managed(&mut self, id: u16, value: Vec<u8>) {
        if let Some(attr) = self.attributes.get_mut(&id) {
            attr.value = value;
        } else {
            self.set_readonly(id, value);
        }
    }

    /// Increment the load count.
    pub fn increment_load_count(&mut self) {
        if let Some(attr) = self.attributes.get_mut(&attr::LOAD_COUNT) {
            let mut count = u32::from_be_bytes(attr.value[..4].try_into().unwrap_or([0; 4]));
            count = count.saturating_add(1);
            attr.value = count.to_be_bytes().to_vec();
        }
    }

    /// Get all attributes for READ ATTRIBUTE response.
    pub fn all_attributes(&self) -> impl Iterator<Item = &MamAttribute> {
        self.attributes.values()
    }

    fn set_readonly(&mut self, id: u16, value: Vec<u8>) {
        self.attributes.insert(
            id,
            MamAttribute {
                id,
                read_only: true,
                value,
            },
        );
    }

    fn set_writable(&mut self, id: u16, value: Vec<u8>) {
        self.attributes.insert(
            id,
            MamAttribute {
                id,
                read_only: false,
                value,
            },
        );
    }
}

impl Default for MamAttributes {
    fn default() -> Self {
        Self::new_for_cartridge("", "IBM", "0000000000")
    }
}

/// Pad or truncate an ASCII string to exactly `len` bytes (space-padded).
fn pad_ascii(s: &str, len: usize) -> Vec<u8> {
    let mut v = vec![0x20u8; len]; // space-fill
    let bytes = s.as_bytes();
    let copy_len = bytes.len().min(len);
    v[..copy_len].copy_from_slice(&bytes[..copy_len]);
    v
}
