//! MOVE MEDIUM (A5h) command handler.

use std::sync::Arc;

use iscsi_target::{MediaLoadNotify, ScsiResult};
use tracing::trace;

use crate::sense::{self, SenseBuilder};
use crate::state::{ChangerState, ELEM_DTE, ELEM_IEE, ELEM_MTE, MediumType};

/// Handle MOVE MEDIUM (A5h).
pub fn handle_move_medium(
    cdb: &[u8],
    st: &mut ChangerState,
    drives: &[Arc<dyn MediaLoadNotify>],
) -> ScsiResult {
    let _transport = ((cdb[2] as u16) << 8) | cdb[3] as u16;
    let source = ((cdb[4] as u16) << 8) | cdb[5] as u16;
    let dest = ((cdb[6] as u16) << 8) | cdb[7] as u16;
    let invert = cdb[10] & 0x01 != 0;

    // No media inversion support
    if invert {
        return SenseBuilder::invalid_field_in_cdb().to_check_condition();
    }

    // Validate source element exists
    if !st.elements.contains_key(&source) {
        return SenseBuilder::invalid_element_address()
            .with_information(source as u32)
            .to_check_condition();
    }

    // Validate destination element exists
    if !st.elements.contains_key(&dest) {
        return SenseBuilder::invalid_element_address()
            .with_information(dest as u32)
            .to_check_condition();
    }

    // Cannot move TO medium transport element
    if st.elements[&dest].element_type == ELEM_MTE {
        return SenseBuilder::invalid_element_address()
            .with_information(dest as u32)
            .to_check_condition();
    }

    // Check source has media
    if !st.elements[&source].full {
        return SenseBuilder::source_element_empty()
            .with_information(source as u32)
            .to_check_condition();
    }

    // Check destination is empty
    if st.elements[&dest].full {
        return SenseBuilder::destination_element_full()
            .with_information(dest as u32)
            .to_check_condition();
    }

    // Check source element is accessible
    if st.elements[&source].disabled {
        return SenseBuilder::element_disabled()
            .with_information(source as u32)
            .to_check_condition();
    }

    // Check destination element is accessible
    if st.elements[&dest].disabled {
        return SenseBuilder::element_disabled()
            .with_information(dest as u32)
            .to_check_condition();
    }

    // Check medium removal prevention for I/E destinations
    if st.elements[&dest].element_type == ELEM_IEE && st.prevent_medium_removal {
        return SenseBuilder::medium_removal_prevented().to_check_condition();
    }

    // Check drive not installed for DTE
    if st.elements[&source].element_type == ELEM_DTE && !st.elements[&source].access {
        return SenseBuilder::drive_not_installed()
            .with_information(source as u32)
            .to_check_condition();
    }
    if st.elements[&dest].element_type == ELEM_DTE && !st.elements[&dest].access {
        return SenseBuilder::drive_not_installed()
            .with_information(dest as u32)
            .to_check_condition();
    }

    // Notify source drive (if DTE) that media is being unloaded
    if st.elements[&source].element_type == ELEM_DTE {
        if let Some(drive_idx) = st.drive_index(source) {
            if let Some(drive) = drives.get(drive_idx) {
                drive.media_unloaded();
            }
        }
    }

    // Transfer the media
    let barcode = st.elements.get_mut(&source).unwrap().barcode.take();
    let medium_type = st.elements[&source].medium_type;
    st.elements.get_mut(&source).unwrap().full = false;
    st.elements.get_mut(&source).unwrap().medium_type = MediumType::Data;
    st.elements.get_mut(&source).unwrap().source_element = 0;

    {
        let dst_elem = st.elements.get_mut(&dest).unwrap();
        dst_elem.full = true;
        dst_elem.barcode.clone_from(&barcode);
        dst_elem.source_element = source;
        dst_elem.medium_type = medium_type;
        dst_elem.import_export = false; // Moved by robot, not operator
    }

    // Notify destination drive (if DTE) that media is loaded
    if st.elements[&dest].element_type == ELEM_DTE {
        if let Some(drive_idx) = st.drive_index(dest) {
            if let Some(drive) = drives.get(drive_idx) {
                if let Some(ref bc) = barcode {
                    drive.media_loaded(bc);
                }
            }
        }
    }

    st.total_moves += 1;
    st.picker_position = dest;

    trace!(source, dest, barcode = ?barcode, "MOVE MEDIUM complete");

    sense::good()
}
