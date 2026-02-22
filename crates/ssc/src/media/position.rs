//! Logical and physical tape position models.

use super::geometry::TapeGeometry;
use serde::Serialize;

/// Logical tape position as seen by the SCSI initiator.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LogicalPosition {
    /// Active partition (0-3).
    pub partition: u8,
    /// Logical Object Identifier — count of blocks+filemarks from BOP.
    pub block_number: u64,
    /// Count of filemarks from BOP to current position.
    pub file_number: u64,
}

impl LogicalPosition {
    pub fn at_bop(&self) -> bool {
        self.block_number == 0
    }
}

/// Simulated physical tape position for visualization.
#[derive(Debug, Clone, Serialize)]
pub struct PhysicalPosition {
    /// Current wrap number (0-based).
    pub wrap: u32,
    /// Position within current wrap as fraction (0.0 = start, 1.0 = end).
    pub offset_in_wrap_pct: f64,
}

impl Default for PhysicalPosition {
    fn default() -> Self {
        Self {
            wrap: 0,
            offset_in_wrap_pct: 0.0,
        }
    }
}

/// Map a logical position to a plausible physical position.
///
/// This does not need to be bit-accurate to real hardware — it just needs to
/// produce realistic wrap numbers and positions so that wrap transitions,
/// backhitch estimation, and dashboard display look reasonable.
pub fn logical_to_physical(
    block_number: u64,
    total_blocks: u64,
    geometry: &TapeGeometry,
) -> PhysicalPosition {
    if total_blocks == 0 || block_number == 0 {
        return PhysicalPosition::default();
    }

    // Fraction of tape used
    let frac = (block_number as f64) / (total_blocks.max(1) as f64);
    let total_wraps = geometry.num_wraps as f64;

    // Linear mapping: fraction of capacity → fraction of wraps
    let wrap_position = frac * total_wraps;
    let wrap = (wrap_position.floor() as u32).min(geometry.num_wraps.saturating_sub(1));
    let offset = wrap_position.fract();

    // Serpentine: even wraps go forward, odd wraps go backward
    let offset_in_wrap_pct = if wrap % 2 == 0 { offset } else { 1.0 - offset };

    PhysicalPosition {
        wrap,
        offset_in_wrap_pct,
    }
}

/// Estimate the physical distance (in wraps) between two positions.
/// Used for seek time estimation and backhitch detection.
pub fn wrap_distance(from: &PhysicalPosition, to: &PhysicalPosition) -> u32 {
    (from.wrap as i64 - to.wrap as i64).unsigned_abs() as u32
}

/// Returns true if moving from `from` to `to` requires a direction reversal
/// (backhitch) within the same wrap.
pub fn requires_backhitch(from: &PhysicalPosition, to: &PhysicalPosition) -> bool {
    if from.wrap != to.wrap {
        return true; // Cross-wrap always involves repositioning
    }
    // Same wrap: check if we need to go backward
    let forward = from.wrap % 2 == 0;
    if forward {
        to.offset_in_wrap_pct < from.offset_in_wrap_pct
    } else {
        to.offset_in_wrap_pct > from.offset_in_wrap_pct
    }
}
