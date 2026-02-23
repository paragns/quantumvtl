pub mod login;
pub mod pdu;
pub mod session;
pub mod target;

pub use session::{SessionInfo, SessionRegistry};

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

/// Notification interface for changer-drive coordination.
///
/// When the media changer moves a tape into or out of a drive element,
/// it calls these methods so the drive can update its internal state.
pub trait MediaLoadNotify: Send + Sync {
    fn media_loaded(&self, barcode: &str);
    fn media_unloaded(&self);
}
