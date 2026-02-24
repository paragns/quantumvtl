//! Drive event types for WebSocket real-time updates.

use serde::Serialize;

use crate::snapshot::DriveActivity;

/// Structured event emitted by a tape drive for real-time dashboard updates.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DriveEvent {
    /// Drive activity state changed.
    StateChange { drive: u8, state: DriveActivity },
    /// Tape position updated.
    PositionUpdate {
        drive: u8,
        block: u64,
        wrap: u32,
        wrap_pct: f64,
    },
    /// Buffer state updated.
    BufferUpdate {
        drive: u8,
        write_pct: f64,
        read_pct: f64,
        state: String,
    },
    /// Backhitch occurred (tape direction reversal).
    Backhitch {
        drive: u8,
        from_wrap: u32,
        to_wrap: u32,
    },
    /// A SCSI operation completed.
    Operation {
        drive: u8,
        opcode: u8,
        name: String,
        duration_us: u64,
    },
    /// Media event (load, unload, etc.).
    MediaEvent {
        drive: u8,
        event: String,
        barcode: String,
    },
}
