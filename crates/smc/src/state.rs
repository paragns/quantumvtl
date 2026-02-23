//! Changer state: element map, library status, Unit Attention queue.

use std::collections::BTreeMap;
use serde::Serialize;

// ── Element addressing (Quantum Scalar spec-mandated) ──────────────────────

/// Well-known starting addresses per the Quantum Scalar SCSI specification.
pub const MTE_START: u16 = 0x0001;
pub const IEE_START: u16 = 0x0010;
pub const DTE_START: u16 = 0x0100;
pub const STE_START: u16 = 0x1000;

// ── Element types ──────────────────────────────────────────────────────────

/// SCSI element type codes.
pub const ELEM_MTE: u8 = 1; // Medium Transport Element (robot arm)
pub const ELEM_STE: u8 = 2; // Storage Element (slot)
pub const ELEM_IEE: u8 = 3; // Import/Export Element (mailslot)
pub const ELEM_DTE: u8 = 4; // Data Transfer Element (drive)

// ── Medium type classification ─────────────────────────────────────────────

/// Medium type as reported in element descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MediumType {
    /// Data cartridge (default).
    Data = 0,
    /// Cleaning cartridge.
    Cleaning = 1,
    /// Diagnostic cartridge.
    Diagnostic = 3,
    /// WORM cartridge.
    Worm = 4,
    /// Microcode/firmware cartridge.
    Microcode = 5,
}

impl MediumType {
    /// Classify from barcode suffix conventions.
    pub fn from_barcode(barcode: &str) -> Self {
        let upper = barcode.to_uppercase();
        if upper.starts_with("CLN") {
            MediumType::Cleaning
        } else if upper.starts_with("DG") || upper.starts_with("DIAG") {
            MediumType::Diagnostic
        } else {
            // Could detect WORM from barcode suffix in future
            MediumType::Data
        }
    }

    /// Three-bit encoding for element descriptor byte 9 bits 2-0.
    pub fn to_bits(self) -> u8 {
        self as u8
    }
}

impl Default for MediumType {
    fn default() -> Self {
        MediumType::Data
    }
}

// ── Element ────────────────────────────────────────────────────────────────

/// A single library element (slot, drive, picker, or mailslot).
#[derive(Debug, Clone)]
pub struct Element {
    /// Element type code (ELEM_MTE, ELEM_STE, ELEM_IEE, ELEM_DTE).
    pub element_type: u8,
    /// Whether this element contains media.
    pub full: bool,
    /// Barcode label of the media, if present.
    pub barcode: Option<String>,
    /// Source element address (where this media came from).
    pub source_element: u16,
    /// Medium type classification.
    pub medium_type: MediumType,
    /// Whether the medium transport can access this element.
    pub access: bool,
    /// Whether this element is in an abnormal state.
    pub except: bool,
    /// Whether this element is disabled (e.g., magazine removed, drive offline).
    pub disabled: bool,
    /// ASC/ASCQ for exception condition, if except=true.
    pub asc_ascq: Option<(u8, u8)>,
    /// I/E only: media was placed by operator (not robot).
    pub import_export: bool,
    /// I/E only: operator intervention required.
    pub operator_intervention: bool,
}

impl Element {
    /// Create a new empty element.
    pub fn new(element_type: u8) -> Self {
        Self {
            element_type,
            full: false,
            barcode: None,
            source_element: 0,
            medium_type: MediumType::Data,
            access: true,
            except: false,
            disabled: false,
            asc_ascq: None,
            import_export: false,
            operator_intervention: false,
        }
    }

    /// Create a storage element pre-loaded with a cartridge.
    pub fn new_with_media(element_type: u8, barcode: &str) -> Self {
        let medium_type = MediumType::from_barcode(barcode);
        Self {
            element_type,
            full: true,
            barcode: Some(barcode.to_string()),
            source_element: 0,
            medium_type,
            access: true,
            except: false,
            disabled: false,
            asc_ascq: None,
            import_export: false,
            operator_intervention: false,
        }
    }
}

// ── Library state ──────────────────────────────────────────────────────────

/// Overall library readiness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum LibraryState {
    /// Scanning inventory after power-on.
    Initializing,
    /// Normal operation.
    Ready,
    /// Not ready for motion (with reason).
    NotReady(String),
    /// Moving media.
    Moving {
        source: u16,
        dest: u16,
    },
    /// Scanning inventory.
    Scanning,
}

impl LibraryState {
    pub fn is_ready(&self) -> bool {
        matches!(self, LibraryState::Ready)
    }
}

// ── Unit Attention ─────────────────────────────────────────────────────────

/// A queued Unit Attention condition.
#[derive(Debug, Clone)]
pub struct UnitAttention {
    pub sense_key: u8,
    pub asc: u8,
    pub ascq: u8,
}

impl UnitAttention {
    pub fn power_on_reset() -> Self {
        Self { sense_key: 0x06, asc: 0x29, ascq: 0x00 }
    }

    pub fn element_status_changed() -> Self {
        Self { sense_key: 0x06, asc: 0x28, ascq: 0x00 }
    }

    pub fn mode_parameters_changed() -> Self {
        Self { sense_key: 0x06, asc: 0x2A, ascq: 0x01 }
    }

    pub fn door_opened_closed() -> Self {
        Self { sense_key: 0x06, asc: 0x28, ascq: 0x01 }
    }

    pub fn firmware_changed() -> Self {
        Self { sense_key: 0x06, asc: 0x3F, ascq: 0x01 }
    }
}

// ── Changer State ──────────────────────────────────────────────────────────

/// Internal changer state protected by a mutex.
#[derive(Debug)]
pub struct ChangerState {
    /// Element address ranges.
    pub start_picker: u16,
    pub num_pickers: u16,
    pub start_drive: u16,
    pub num_drives: u16,
    pub start_slot: u16,
    pub num_slots: u16,
    pub start_iee: u16,
    pub num_iee: u16,

    /// Sparse element map keyed by element address.
    pub elements: BTreeMap<u16, Element>,

    /// Library readiness state.
    pub library_state: LibraryState,

    /// Pending Unit Attention conditions (FIFO).
    pub ua_queue: Vec<UnitAttention>,

    /// Whether medium removal to I/E is prevented.
    pub prevent_medium_removal: bool,

    /// Total move operations since startup.
    pub total_moves: u64,

    /// Current picker position (element address the picker is near).
    pub picker_position: u16,

    /// Simulated temperature (Celsius).
    pub temperature_c: u8,

    /// Simulated humidity (percent).
    pub humidity_pct: u8,
}

impl ChangerState {
    /// Create a new changer state with spec-mandated element addresses.
    pub fn new(
        num_drives: u16,
        num_slots: u16,
        num_iee: u16,
        media_barcodes: &[String],
    ) -> Self {
        let mut elements = BTreeMap::new();

        // MTE at 0x0001
        elements.insert(MTE_START, Element::new(ELEM_MTE));

        // I/E elements starting at 0x0010
        for i in 0..num_iee {
            elements.insert(IEE_START + i, Element::new(ELEM_IEE));
        }

        // DTE elements starting at 0x0100
        for i in 0..num_drives {
            elements.insert(DTE_START + i, Element::new(ELEM_DTE));
        }

        // STE elements starting at 0x1000
        for i in 0..num_slots {
            let addr = STE_START + i;
            if let Some(barcode) = media_barcodes.get(i as usize) {
                elements.insert(addr, Element::new_with_media(ELEM_STE, barcode));
            } else {
                elements.insert(addr, Element::new(ELEM_STE));
            }
        }

        Self {
            start_picker: MTE_START,
            num_pickers: 1,
            start_drive: DTE_START,
            num_drives,
            start_slot: STE_START,
            num_slots,
            start_iee: IEE_START,
            num_iee,
            elements,
            library_state: LibraryState::Ready,
            ua_queue: Vec::new(),
            prevent_medium_removal: false,
            total_moves: 0,
            picker_position: MTE_START,
            temperature_c: 22,
            humidity_pct: 45,
        }
    }

    /// Dequeue the next pending Unit Attention, if any.
    pub fn pop_unit_attention(&mut self) -> Option<UnitAttention> {
        if self.ua_queue.is_empty() {
            None
        } else {
            Some(self.ua_queue.remove(0))
        }
    }

    /// Queue a Unit Attention condition.
    pub fn push_unit_attention(&mut self, ua: UnitAttention) {
        self.ua_queue.push(ua);
    }

    /// Get the drive index (0-based) from an element address.
    pub fn drive_index(&self, addr: u16) -> Option<usize> {
        if addr >= self.start_drive && addr < self.start_drive + self.num_drives {
            Some((addr - self.start_drive) as usize)
        } else {
            None
        }
    }
}
