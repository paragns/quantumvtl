//! WebSocket event types for the media changer.

use serde::Serialize;

/// Events emitted by the changer for real-time dashboard updates.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ChangerEvent {
    /// A MOVE MEDIUM operation has started.
    MoveStarted {
        source: u16,
        dest: u16,
        barcode: Option<String>,
    },
    /// A MOVE MEDIUM operation completed successfully.
    MoveCompleted {
        source: u16,
        dest: u16,
        barcode: Option<String>,
        duration_ms: u64,
    },
    /// A MOVE MEDIUM operation failed.
    MoveFailed {
        source: u16,
        dest: u16,
        reason: String,
    },
    /// An inventory scan has started.
    InventoryScanStarted,
    /// An inventory scan completed.
    InventoryScanCompleted {
        elements_scanned: u32,
    },
    /// A TapeAlert flag was raised.
    AlertRaised {
        flag: u16,
        severity: AlertSeverity,
        description: String,
    },
    /// Library state changed.
    StateChanged {
        old: String,
        new: String,
    },
    /// Picker position updated.
    PickerMoved {
        from_address: u16,
        to_address: u16,
    },
}

/// TapeAlert severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AlertSeverity {
    Informational,
    Warning,
    Critical,
}
