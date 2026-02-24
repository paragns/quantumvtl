//! Dashboard snapshot types for the media changer.

use crate::state::{self, ChangerState, LibraryState, MediumType};
use serde::Serialize;

/// Public element type enum for API consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ElementType {
    Transport,
    Storage,
    ImportExport,
    DataTransfer,
}

impl ElementType {
    pub fn from_scsi(code: u8) -> Self {
        match code {
            state::ELEM_MTE => ElementType::Transport,
            state::ELEM_IEE => ElementType::ImportExport,
            state::ELEM_DTE => ElementType::DataTransfer,
            _ => ElementType::Storage,
        }
    }
}

/// Snapshot of a single element for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct ElementSnapshot {
    pub address: u16,
    pub element_type: ElementType,
    pub full: bool,
    pub barcode: Option<String>,
    pub source_element: u16,
    /// Whether the medium transport can access this element.
    pub access: bool,
    /// Whether this element is in an abnormal state.
    pub except: bool,
    /// Whether this element is disabled.
    pub disabled: bool,
    /// ASC/ASCQ for exception condition.
    pub asc_ascq: Option<(u8, u8)>,
    /// Medium type classification.
    pub medium_type: MediumType,
    /// I/E: media was placed by operator.
    pub import_export: bool,
    /// I/E: operator intervention required.
    pub operator_intervention: bool,
}

/// Snapshot of the entire changer for the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct ChangerSnapshot {
    pub vendor: String,
    pub product: String,
    pub serial: String,
    pub firmware_version: String,
    pub num_drives: u16,
    pub num_slots: u16,
    pub num_import_export: u16,
    pub elements: Vec<ElementSnapshot>,
    /// Library readiness state.
    pub state: LibraryState,
    /// Simulated temperature (Celsius).
    pub temperature_c: u8,
    /// Simulated humidity (percent).
    pub humidity_pct: u8,
    /// Total move operations since startup.
    pub total_moves: u64,
    /// Current picker position (element address).
    pub picker_position: u16,
    /// Active TapeAlert flag numbers.
    pub active_alerts: Vec<u16>,
    /// Whether medium removal to I/E is prevented.
    pub prevent_medium_removal: bool,
}

impl ChangerSnapshot {
    /// Build a snapshot from changer state.
    pub fn from_state(st: &ChangerState, vendor: &str, product: &str, serial: &str) -> Self {
        let elements = st
            .elements
            .iter()
            .map(|(&addr, elem)| ElementSnapshot {
                address: addr,
                element_type: ElementType::from_scsi(elem.element_type),
                full: elem.full,
                barcode: elem.barcode.clone(),
                source_element: elem.source_element,
                access: elem.access,
                except: elem.except,
                disabled: elem.disabled,
                asc_ascq: elem.asc_ascq,
                medium_type: elem.medium_type,
                import_export: elem.import_export,
                operator_intervention: elem.operator_intervention,
            })
            .collect();

        Self {
            vendor: vendor.trim().to_string(),
            product: product.trim().to_string(),
            serial: serial.to_string(),
            firmware_version: "0100".to_string(),
            num_drives: st.num_drives,
            num_slots: st.num_slots,
            num_import_export: st.num_iee,
            elements,
            state: st.library_state.clone(),
            temperature_c: st.temperature_c,
            humidity_pct: st.humidity_pct,
            total_moves: st.total_moves,
            picker_position: st.picker_position,
            active_alerts: Vec::new(),
            prevent_medium_removal: st.prevent_medium_removal,
        }
    }
}
