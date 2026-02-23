//! Control commands: TUR, REQUEST SENSE, PREVENT/ALLOW, INIT ELEMENT STATUS.

use crate::sense::{self, SenseBuilder};
use crate::state::{ChangerState, LibraryState};
use iscsi_target::ScsiResult;

/// Handle TEST UNIT READY (00h).
pub fn handle_test_unit_ready(st: &mut ChangerState) -> ScsiResult {
    // Check for pending Unit Attention first
    if let Some(ua) = st.pop_unit_attention() {
        return SenseBuilder::new(ua.sense_key, ua.asc, ua.ascq).to_check_condition();
    }

    match &st.library_state {
        LibraryState::Ready => sense::good(),
        LibraryState::Initializing | LibraryState::Scanning => {
            SenseBuilder::becoming_ready().to_check_condition()
        }
        LibraryState::Moving { .. } => {
            // Library is busy but operational
            sense::good()
        }
        LibraryState::NotReady(reason) => {
            if reason.contains("manual") || reason.contains("intervention") {
                SenseBuilder::not_ready_manual_intervention().to_check_condition()
            } else if reason.contains("offline") {
                SenseBuilder::not_ready_offline().to_check_condition()
            } else {
                SenseBuilder::not_ready().to_check_condition()
            }
        }
    }
}

/// Handle REQUEST SENSE (03h).
pub fn handle_request_sense(cdb: &[u8], st: &mut ChangerState) -> ScsiResult {
    let alloc_len = cdb[4] as usize;

    // If there's a pending Unit Attention, return it
    let data = if let Some(ua) = st.pop_unit_attention() {
        SenseBuilder::new(ua.sense_key, ua.asc, ua.ascq).build()
    } else {
        // No pending sense — return NO SENSE
        SenseBuilder::new(0x00, 0x00, 0x00).build()
    };

    let mut data = data;
    data.truncate(alloc_len);
    sense::good_with_data(data)
}

/// Handle PREVENT ALLOW MEDIUM REMOVAL (1Eh).
pub fn handle_prevent_allow_medium_removal(cdb: &[u8], st: &mut ChangerState) -> ScsiResult {
    let prevent = cdb[4] & 0x03;
    let _preempt = cdb[4] & 0x80 != 0;

    match prevent {
        0x00 => {
            // Allow medium removal
            st.prevent_medium_removal = false;
        }
        0x01 => {
            // Prevent medium removal
            st.prevent_medium_removal = true;
        }
        _ => {
            // Values 02h and 03h not supported
            return SenseBuilder::invalid_field_in_cdb().to_check_condition();
        }
    }

    sense::good()
}

/// Handle INITIALIZE ELEMENT STATUS (07h).
pub fn handle_initialize_element_status(_cdb: &[u8]) -> ScsiResult {
    // In our simulation, element status is always known.
    // Accept and return GOOD immediately.
    sense::good()
}

/// Handle INITIALIZE ELEMENT STATUS WITH RANGE (E7h).
pub fn handle_init_element_status_with_range(_cdb: &[u8]) -> ScsiResult {
    // Same as above — our simulation always has current state.
    // The Range, Fast, NBL bits don't change behavior for us.
    sense::good()
}
