//! SCSI command handlers for the media changer.

pub mod control;
pub mod element_status;
pub mod inquiry;
pub mod log;
pub mod mode;
pub mod move_medium;
pub mod report_luns;

/// SCSI opcode constants for the media changer.
pub mod opcodes {
    pub const TEST_UNIT_READY: u8 = 0x00;
    pub const REQUEST_SENSE: u8 = 0x03;
    pub const INITIALIZE_ELEMENT_STATUS: u8 = 0x07;
    pub const INQUIRY: u8 = 0x12;
    pub const MODE_SELECT_6: u8 = 0x15;
    pub const RESERVE_ELEMENT_6: u8 = 0x16;
    pub const RELEASE_ELEMENT_6: u8 = 0x17;
    pub const MODE_SENSE_6: u8 = 0x1A;
    pub const SEND_DIAGNOSTIC: u8 = 0x1D;
    pub const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
    pub const POSITION_TO_ELEMENT: u8 = 0x2B;
    pub const READ_BUFFER: u8 = 0x3C;
    pub const WRITE_BUFFER: u8 = 0x3B;
    pub const LOG_SENSE: u8 = 0x4D;
    pub const MODE_SELECT_10: u8 = 0x55;
    pub const RELEASE_ELEMENT_10: u8 = 0x57;
    pub const MODE_SENSE_10: u8 = 0x5A;
    pub const PERSISTENT_RESERVE_IN: u8 = 0x5E;
    pub const PERSISTENT_RESERVE_OUT: u8 = 0x5F;
    pub const INIT_ELEMENT_STATUS_WITH_RANGE: u8 = 0xE7;
    pub const REPORT_LUNS: u8 = 0xA0;
    pub const MOVE_MEDIUM: u8 = 0xA5;
    pub const EXCHANGE_MEDIUM: u8 = 0xA6;
    pub const READ_ELEMENT_STATUS: u8 = 0xB8;
    pub const RESERVE_ELEMENT_10: u8 = 0x56;
    pub const SEND_VOLUME_TAG: u8 = 0xB6;
    pub const REQUEST_VOLUME_ELEMENT_ADDRESS: u8 = 0xB5;
}
