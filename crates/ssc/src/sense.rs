//! SCSI sense data builder for fixed-format (70h) and descriptor-format (72h) responses.

/// Sense data builder with fluent API.
#[derive(Debug, Clone)]
pub struct SenseBuilder {
    response_code: u8, // 0x70 = current fixed, 0x72 = current descriptor
    sense_key: u8,
    asc: u8,
    ascq: u8,
    information: Option<u32>,
    cmd_specific: Option<u32>,
    filemark: bool,
    eom: bool,
    ili: bool,
    valid: bool,
}

impl SenseBuilder {
    // --- Constructors for common sense conditions ---

    pub fn new(sense_key: u8, asc: u8, ascq: u8) -> Self {
        Self {
            response_code: 0x70,
            sense_key,
            asc,
            ascq,
            information: None,
            cmd_specific: None,
            filemark: false,
            eom: false,
            ili: false,
            valid: false,
        }
    }

    pub fn no_sense() -> Self {
        Self::new(0x00, 0x00, 0x00)
    }

    /// NOT READY: MEDIUM NOT PRESENT
    pub fn no_media() -> Self {
        Self::new(0x02, 0x3A, 0x00)
    }

    /// NOT READY: LOGICAL UNIT NOT READY, OPERATION IN PROGRESS
    pub fn not_ready_in_progress() -> Self {
        Self::new(0x02, 0x04, 0x07)
    }

    /// NOT READY: LOGICAL UNIT IS IN PROCESS OF BECOMING READY
    pub fn not_ready_becoming_ready() -> Self {
        Self::new(0x02, 0x04, 0x01)
    }

    /// ILLEGAL REQUEST: INVALID COMMAND OPERATION CODE
    pub fn invalid_opcode() -> Self {
        Self::new(0x05, 0x20, 0x00)
    }

    /// ILLEGAL REQUEST: INVALID FIELD IN CDB
    pub fn invalid_field_in_cdb() -> Self {
        Self::new(0x05, 0x24, 0x00)
    }

    /// ILLEGAL REQUEST: INVALID FIELD IN PARAMETER LIST
    pub fn invalid_field_in_parameter_list() -> Self {
        Self::new(0x05, 0x26, 0x00)
    }

    /// ILLEGAL REQUEST: LOGICAL UNIT NOT SUPPORTED
    pub fn lun_not_supported() -> Self {
        Self::new(0x05, 0x25, 0x00)
    }

    /// BLANK CHECK: END-OF-DATA DETECTED
    pub fn end_of_data() -> Self {
        Self::new(0x08, 0x00, 0x05)
    }

    /// NO SENSE: FILEMARK DETECTED (with filemark flag)
    pub fn filemark_detected() -> Self {
        Self::new(0x00, 0x00, 0x01).with_filemark()
    }

    /// NO SENSE: BEGINNING-OF-PARTITION/MEDIUM DETECTED
    pub fn beginning_of_partition() -> Self {
        Self::new(0x00, 0x00, 0x04)
    }

    /// NO SENSE: END-OF-PARTITION/MEDIUM DETECTED (early warning)
    pub fn end_of_partition() -> Self {
        Self::new(0x00, 0x00, 0x02).with_eom()
    }

    /// UNIT ATTENTION with specified ASC/ASCQ
    pub fn unit_attention(asc: u8, ascq: u8) -> Self {
        Self::new(0x06, asc, ascq)
    }

    /// DATA PROTECT: WRITE PROTECTED
    pub fn write_protected() -> Self {
        Self::new(0x07, 0x27, 0x00)
    }

    /// VOLUME OVERFLOW
    pub fn volume_overflow() -> Self {
        Self::new(0x0D, 0x00, 0x02).with_eom()
    }

    /// MEDIUM ERROR — unrecoverable I/O error on the medium (sense key 0x03).
    pub fn medium_error() -> Self {
        // ASC/ASCQ 0x11/0x00 = Unrecovered read error (generic medium error)
        Self::new(0x03, 0x11, 0x00)
    }

    /// MEDIUM ERROR — write fault (sense key 0x03, ASC/ASCQ 0x03/0x00).
    pub fn write_fault() -> Self {
        Self::new(0x03, 0x03, 0x00)
    }

    // --- Builder methods ---

    pub fn with_information(mut self, info: u32) -> Self {
        self.information = Some(info);
        self.valid = true;
        self
    }

    pub fn with_cmd_specific(mut self, info: u32) -> Self {
        self.cmd_specific = Some(info);
        self
    }

    pub fn with_filemark(mut self) -> Self {
        self.filemark = true;
        self
    }

    pub fn with_eom(mut self) -> Self {
        self.eom = true;
        self
    }

    pub fn with_ili(mut self) -> Self {
        self.ili = true;
        self
    }

    // --- Build ---

    /// Build an 18-byte fixed-format (70h) sense data buffer.
    pub fn build(&self) -> Vec<u8> {
        let mut sense = vec![0u8; 18];

        // Byte 0: VALID (bit 7) | RESPONSE CODE (bits 6-0)
        sense[0] = self.response_code;
        if self.valid {
            sense[0] |= 0x80;
        }

        // Byte 1: Segment number (obsolete, always 0)

        // Byte 2: FILEMARK (bit 7) | EOM (bit 6) | ILI (bit 5) | SENSE KEY (bits 3-0)
        sense[2] = self.sense_key & 0x0F;
        if self.filemark {
            sense[2] |= 0x80;
        }
        if self.eom {
            sense[2] |= 0x40;
        }
        if self.ili {
            sense[2] |= 0x20;
        }

        // Bytes 3-6: INFORMATION field (big-endian)
        if let Some(info) = self.information {
            sense[3] = ((info >> 24) & 0xFF) as u8;
            sense[4] = ((info >> 16) & 0xFF) as u8;
            sense[5] = ((info >> 8) & 0xFF) as u8;
            sense[6] = (info & 0xFF) as u8;
        }

        // Byte 7: ADDITIONAL SENSE LENGTH (bytes remaining after this byte)
        sense[7] = 10;

        // Bytes 8-11: COMMAND-SPECIFIC INFORMATION
        if let Some(csi) = self.cmd_specific {
            sense[8] = ((csi >> 24) & 0xFF) as u8;
            sense[9] = ((csi >> 16) & 0xFF) as u8;
            sense[10] = ((csi >> 8) & 0xFF) as u8;
            sense[11] = (csi & 0xFF) as u8;
        }

        // Byte 12: ADDITIONAL SENSE CODE
        sense[12] = self.asc;
        // Byte 13: ADDITIONAL SENSE CODE QUALIFIER
        sense[13] = self.ascq;

        // Bytes 14-17: FRU code, sense key specific (zeros for now)

        sense
    }
}

/// Convenience: build CHECK CONDITION result from SenseBuilder
impl SenseBuilder {
    pub fn to_check_condition(&self) -> crate::ScsiResult {
        crate::ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: self.build(),
        }
    }

    pub fn to_check_condition_with_data(&self, data: Vec<u8>) -> crate::ScsiResult {
        crate::ScsiResult {
            status: 0x02,
            data_in: data,
            sense: self.build(),
        }
    }
}

/// Quick GOOD status result
pub fn good() -> crate::ScsiResult {
    crate::ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// Quick GOOD status with data-in
pub fn good_with_data(data: Vec<u8>) -> crate::ScsiResult {
    crate::ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}
