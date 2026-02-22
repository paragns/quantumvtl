pub mod login;
pub mod pdu;
pub mod target;

/// Result of executing a SCSI command.
pub struct ScsiResult {
    /// SCSI status byte: 0x00 = GOOD, 0x02 = CHECK CONDITION.
    pub status: u8,
    /// Data-In payload (response data for READ-type commands).
    pub data_in: Vec<u8>,
    /// Sense data (for CHECK CONDITION status).
    pub sense: Vec<u8>,
}

/// Trait for a SCSI logical unit that can process CDBs.
pub trait ScsiDevice: Send + Sync {
    fn execute_command(&self, cdb: &[u8], data_out: &[u8]) -> ScsiResult;
}
