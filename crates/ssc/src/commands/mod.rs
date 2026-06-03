pub mod attribute;
pub mod control;
pub mod density;
pub mod erase;
pub mod filemarks;
pub mod inquiry;
pub mod log;
pub mod maintenance;
pub mod mode;
pub mod position;
pub mod read;
pub mod write;

/// SCSI command opcodes — SSC (tape drive) commands.
pub mod opcodes {
    pub const TEST_UNIT_READY: u8 = 0x00;
    pub const REWIND: u8 = 0x01;
    pub const REQUEST_SENSE: u8 = 0x03;
    pub const FORMAT_MEDIUM: u8 = 0x04;
    pub const READ_BLOCK_LIMITS: u8 = 0x05;
    pub const READ_BUFFER: u8 = 0x3C;
    pub const READ_6: u8 = 0x08;
    pub const WRITE_6: u8 = 0x0A;
    pub const SET_CAPACITY: u8 = 0x0B;
    pub const WRITE_FILEMARKS_6: u8 = 0x10;
    pub const SPACE_6: u8 = 0x11;
    pub const INQUIRY: u8 = 0x12;
    pub const VERIFY_6: u8 = 0x13;
    pub const MODE_SELECT_6: u8 = 0x15;
    pub const ERASE_6: u8 = 0x19;
    pub const MODE_SENSE_6: u8 = 0x1A;
    pub const LOAD_UNLOAD: u8 = 0x1B;
    pub const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
    pub const LOCATE_10: u8 = 0x2B;
    pub const READ_POSITION: u8 = 0x34;
    pub const REPORT_DENSITY_SUPPORT: u8 = 0x44;
    pub const LOG_SELECT: u8 = 0x4C;
    pub const LOG_SENSE: u8 = 0x4D;
    pub const MODE_SELECT_10: u8 = 0x55;
    pub const MODE_SENSE_10: u8 = 0x5A;
    pub const PERSISTENT_RESERVE_IN: u8 = 0x5E;
    pub const PERSISTENT_RESERVE_OUT: u8 = 0x5F;
    pub const ALLOW_OVERWRITE: u8 = 0x82;
    pub const READ_ATTRIBUTE: u8 = 0x8C;
    pub const WRITE_ATTRIBUTE: u8 = 0x8D;
    pub const SPACE_16: u8 = 0x91;
    pub const LOCATE_16: u8 = 0x92;
    pub const MAINTENANCE_IN: u8 = 0xA3;
    pub const MAINTENANCE_OUT: u8 = 0xA4;
    pub const WRITE_FILEMARKS_16: u8 = 0x80; // Actually 0x80 in some forms, check spec
}
