//! Robot timing model for physical simulation.
//!
//! Models realistic timing for picker operations in a Quantum Scalar library.

use serde::Serialize;

/// Timing constants for robot operations.
#[derive(Debug, Clone, Serialize)]
pub struct RobotTimingModel {
    /// Time to pick (grab) a cartridge from a slot (seconds).
    pub pick_sec: f64,
    /// Time to place (insert) a cartridge into a slot (seconds).
    pub place_sec: f64,
    /// Time to load a cartridge into a drive (seconds, includes drive load).
    pub drive_load_sec: f64,
    /// Time to unload a cartridge from a drive (seconds, includes drive eject).
    pub drive_unload_sec: f64,
    /// Horizontal travel speed (slots per second).
    pub travel_speed_slots_per_sec: f64,
    /// Vertical travel speed (rows per second).
    pub travel_speed_rows_per_sec: f64,
    /// I/E door open/close cycle time (seconds).
    pub door_cycle_sec: f64,
    /// Time per slot for inventory scan (seconds).
    pub scan_per_slot_sec: f64,
    /// POSITION TO ELEMENT base time (seconds).
    pub position_base_sec: f64,
}

impl RobotTimingModel {
    /// Timing model for a Quantum Scalar i6 (mid-range library).
    pub fn scalar_i6() -> Self {
        Self {
            pick_sec: 2.0,
            place_sec: 2.0,
            drive_load_sec: 20.0,
            drive_unload_sec: 18.0,
            travel_speed_slots_per_sec: 8.0,
            travel_speed_rows_per_sec: 4.0,
            door_cycle_sec: 5.0,
            scan_per_slot_sec: 0.5,
            position_base_sec: 1.0,
        }
    }

    /// Timing model for a Quantum Scalar i3 (small library).
    pub fn scalar_i3() -> Self {
        Self {
            pick_sec: 1.5,
            place_sec: 1.5,
            drive_load_sec: 18.0,
            drive_unload_sec: 16.0,
            travel_speed_slots_per_sec: 10.0,
            travel_speed_rows_per_sec: 5.0,
            door_cycle_sec: 3.0,
            scan_per_slot_sec: 0.3,
            position_base_sec: 0.5,
        }
    }

    /// Estimate move duration in seconds.
    ///
    /// `slot_distance` is the abstract distance between source and destination
    /// expressed in slot units.
    pub fn estimate_move_sec(
        &self,
        slot_distance: u16,
        source_is_drive: bool,
        dest_is_drive: bool,
    ) -> f64 {
        let travel = slot_distance as f64 / self.travel_speed_slots_per_sec;
        let pick = if source_is_drive {
            self.drive_unload_sec
        } else {
            self.pick_sec
        };
        let place = if dest_is_drive {
            self.drive_load_sec
        } else {
            self.place_sec
        };
        pick + travel + place
    }

    /// Estimate inventory scan duration for the given number of elements.
    pub fn estimate_scan_sec(&self, num_elements: u32) -> f64 {
        num_elements as f64 * self.scan_per_slot_sec
    }
}

impl Default for RobotTimingModel {
    fn default() -> Self {
        Self::scalar_i6()
    }
}
