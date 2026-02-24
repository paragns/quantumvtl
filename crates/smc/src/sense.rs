//! Sense data construction for the media changer.
//!
//! Fixed-format sense data (response code 70h) per SPC-4 and the
//! Quantum Scalar SCSI specification.

use iscsi_target::ScsiResult;

/// Builder for fixed-format sense data.
pub struct SenseBuilder {
    sense_key: u8,
    asc: u8,
    ascq: u8,
    information: Option<u32>,
    /// Sense-key specific: field pointer for ILLEGAL REQUEST.
    field_pointer: Option<(bool, u16)>, // (is_cdb, byte_offset)
}

impl SenseBuilder {
    pub fn new(sense_key: u8, asc: u8, ascq: u8) -> Self {
        Self {
            sense_key,
            asc,
            ascq,
            information: None,
            field_pointer: None,
        }
    }

    /// Set the INFORMATION field (bytes 3-6) — typically an element address.
    pub fn with_information(mut self, info: u32) -> Self {
        self.information = Some(info);
        self
    }

    /// Set field pointer for ILLEGAL REQUEST (SKSV=1).
    pub fn with_field_pointer(mut self, is_cdb: bool, byte_offset: u16) -> Self {
        self.field_pointer = Some((is_cdb, byte_offset));
        self
    }

    /// Build the 18-byte fixed-format sense data.
    pub fn build(&self) -> Vec<u8> {
        let mut sense = vec![0u8; 18];
        // Byte 0: Valid bit | Response code (70h = current, fixed)
        sense[0] = if self.information.is_some() {
            0xF0
        } else {
            0x70
        };
        // Byte 2: Sense key
        sense[2] = self.sense_key & 0x0F;
        // Bytes 3-6: Information field
        if let Some(info) = self.information {
            sense[3] = ((info >> 24) & 0xFF) as u8;
            sense[4] = ((info >> 16) & 0xFF) as u8;
            sense[5] = ((info >> 8) & 0xFF) as u8;
            sense[6] = (info & 0xFF) as u8;
        }
        // Byte 7: Additional sense length (10 = bytes 8-17)
        sense[7] = 10;
        // Byte 12: ASC
        sense[12] = self.asc;
        // Byte 13: ASCQ
        sense[13] = self.ascq;
        // Bytes 15-17: Sense-key specific (field pointer for ILLEGAL REQUEST)
        if let Some((is_cdb, offset)) = self.field_pointer {
            sense[15] = 0x80; // SKSV=1
            if is_cdb {
                sense[15] |= 0x40; // C/D=1 (error in CDB)
            }
            sense[16] = ((offset >> 8) & 0xFF) as u8;
            sense[17] = (offset & 0xFF) as u8;
        }
        sense
    }

    /// Build sense data and wrap in a CHECK CONDITION ScsiResult.
    pub fn to_check_condition(&self) -> ScsiResult {
        ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: self.build(),
        }
    }

    // ── Named constructors ─────────────────────────────────────────────

    // Illegal Request (Sense Key 05h)

    /// Invalid opcode in CDB.
    pub fn invalid_opcode() -> Self {
        Self::new(0x05, 0x20, 0x00)
    }

    /// Invalid element address.
    pub fn invalid_element_address() -> Self {
        Self::new(0x05, 0x21, 0x01)
    }

    /// Invalid field in CDB.
    pub fn invalid_field_in_cdb() -> Self {
        Self::new(0x05, 0x24, 0x00)
    }

    /// Invalid field in parameter list.
    pub fn invalid_field_in_parameter_list() -> Self {
        Self::new(0x05, 0x26, 0x00)
    }

    /// Invalid LUN.
    pub fn invalid_lun() -> Self {
        Self::new(0x05, 0x25, 0x00)
    }

    /// Saving parameters not supported.
    pub fn saving_parameters_not_supported() -> Self {
        Self::new(0x05, 0x39, 0x00)
    }

    /// Parameter list length error.
    pub fn parameter_list_length_error() -> Self {
        Self::new(0x05, 0x1A, 0x00)
    }

    /// Incompatible medium installed.
    pub fn incompatible_medium() -> Self {
        Self::new(0x05, 0x30, 0x00)
    }

    /// Command sequence error.
    pub fn command_sequence_error() -> Self {
        Self::new(0x05, 0x2C, 0x00)
    }

    // Element-specific errors (Sense Key 05h, ASC 3Bh)

    /// Medium destination element full.
    pub fn destination_element_full() -> Self {
        Self::new(0x05, 0x3B, 0x0D)
    }

    /// Medium source element empty.
    pub fn source_element_empty() -> Self {
        Self::new(0x05, 0x3B, 0x0E)
    }

    /// Medium magazine not accessible.
    pub fn magazine_not_accessible() -> Self {
        Self::new(0x05, 0x3B, 0x11)
    }

    /// Medium magazine not installed.
    pub fn magazine_not_installed() -> Self {
        Self::new(0x05, 0x3B, 0x12)
    }

    /// Element disabled.
    pub fn element_disabled() -> Self {
        Self::new(0x05, 0x3B, 0x18)
    }

    /// Data transfer element removed.
    pub fn data_transfer_element_removed() -> Self {
        Self::new(0x05, 0x3B, 0x1A)
    }

    /// Media type doesn't match destination.
    pub fn media_type_mismatch() -> Self {
        Self::new(0x05, 0x3B, 0xA0)
    }

    // Medium removal prevented

    /// Medium removal prevented.
    pub fn medium_removal_prevented() -> Self {
        Self::new(0x05, 0x53, 0x02)
    }

    /// Duplicate volume identifier.
    pub fn duplicate_volume_id() -> Self {
        Self::new(0x05, 0x53, 0x07)
    }

    // Not Ready (Sense Key 02h)

    /// Logical unit not ready, cause not reportable.
    pub fn not_ready() -> Self {
        Self::new(0x02, 0x04, 0x00)
    }

    /// Logical unit is becoming ready.
    pub fn becoming_ready() -> Self {
        Self::new(0x02, 0x04, 0x01)
    }

    /// Not ready, manual intervention required.
    pub fn not_ready_manual_intervention() -> Self {
        Self::new(0x02, 0x04, 0x03)
    }

    /// Not ready, offline.
    pub fn not_ready_offline() -> Self {
        Self::new(0x02, 0x04, 0x12)
    }

    // Hardware Error (Sense Key 04h)

    /// Mechanical positioning error.
    pub fn mechanical_positioning_error() -> Self {
        Self::new(0x04, 0x15, 0x01)
    }

    /// Drive did not load/unload tape.
    pub fn drive_load_failure() -> Self {
        Self::new(0x04, 0x53, 0x00)
    }

    /// Data transfer element not installed.
    pub fn drive_not_installed() -> Self {
        Self::new(0x04, 0x83, 0x04)
    }

    /// Component failure.
    pub fn component_failure() -> Self {
        Self::new(0x04, 0x40, 0x80)
    }

    // Unit Attention (Sense Key 06h)

    /// Not-ready to ready transition; element status may have changed.
    pub fn element_status_changed() -> Self {
        Self::new(0x06, 0x28, 0x00)
    }

    /// I/E station opened or closed.
    pub fn ie_station_changed() -> Self {
        Self::new(0x06, 0x28, 0x01)
    }

    /// Power-on or reset.
    pub fn power_on_reset() -> Self {
        Self::new(0x06, 0x29, 0x00)
    }

    /// Mode parameters changed.
    pub fn mode_parameters_changed() -> Self {
        Self::new(0x06, 0x2A, 0x01)
    }

    // Aborted Command (Sense Key 0Bh)

    /// Aborted command.
    pub fn aborted_command() -> Self {
        Self::new(0x0B, 0x00, 0x00)
    }
}

// ── Helper functions ───────────────────────────────────────────────────────

/// Return a GOOD status with no data.
pub fn good() -> ScsiResult {
    ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// Return a GOOD status with data.
pub fn good_with_data(data: Vec<u8>) -> ScsiResult {
    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// Return a RESERVATION CONFLICT status.
pub fn reservation_conflict() -> ScsiResult {
    ScsiResult {
        status: 0x18,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}
