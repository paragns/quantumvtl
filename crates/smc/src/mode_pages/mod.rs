//! Mode page framework for the media changer.
//!
//! The Quantum Scalar spec defines these media changer mode pages:
//! - 1Ch: Informational Exceptions Control
//! - 1Dh: Element Address Assignment
//! - 1Eh: Transport Geometry Parameters
//! - 1Fh/00h: Device Capabilities
//! - 1Fh/41h: Extended Device Capabilities (i7 RAPTOR only)
//!
//! None of the parameters are changeable per the spec.

/// Page control values.
pub const PC_CURRENT: u8 = 0;
pub const PC_CHANGEABLE: u8 = 1;
pub const PC_DEFAULT: u8 = 2;
pub const PC_SAVED: u8 = 3;

/// A mode page entry.
struct ModePageEntry {
    page_code: u8,
    subpage_code: u8,
    /// Default/current page data (including page header).
    data: Vec<u8>,
}

/// Registry of mode pages.
pub struct ModePageRegistry {
    pages: Vec<ModePageEntry>,
}

impl ModePageRegistry {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Register a mode page with its full data (header + parameters).
    pub fn register(&mut self, page_code: u8, subpage_code: u8, data: Vec<u8>) {
        self.pages.push(ModePageEntry {
            page_code,
            subpage_code,
            data,
        });
    }

    /// Get a single page. Returns None if not found.
    pub fn get_page(&self, page_code: u8, subpage_code: u8, pc: u8) -> Option<Vec<u8>> {
        let entry = self.pages.iter().find(|e| {
            e.page_code == page_code && e.subpage_code == subpage_code
        })?;

        if pc == PC_CHANGEABLE {
            // No changeable parameters — return all zeros (same length)
            Some(vec![0u8; entry.data.len()])
        } else {
            // Current, default, and saved are all the same
            Some(entry.data.clone())
        }
    }

    /// Get all pages (page_code 3Fh). Sorted by page code.
    pub fn get_all_pages(&self, pc: u8) -> Vec<u8> {
        // Group by page code to avoid duplicates for subpages
        let mut result = Vec::new();
        for entry in &self.pages {
            let data = if pc == PC_CHANGEABLE {
                vec![0u8; entry.data.len()]
            } else {
                entry.data.clone()
            };
            result.extend_from_slice(&data);
        }
        result
    }
}

/// Build the default mode page registry with all spec-required pages.
pub fn default_registry(
    start_picker: u16,
    num_pickers: u16,
    start_slot: u16,
    num_slots: u16,
    start_iee: u16,
    num_iee: u16,
    start_drive: u16,
    num_drives: u16,
) -> ModePageRegistry {
    let mut reg = ModePageRegistry::new();

    // ── 1Ch: Informational Exceptions Control ──────────────────────────
    {
        let mut page = vec![0u8; 12];
        page[0] = 0x1C; // Page code, PS=0, SPF=0
        page[1] = 0x0A; // Page length (10)
        page[3] = 0x08; // Dexcpt=1 (disable exceptions — must poll)
        // MRIE=0, Interval Timer=0, Report Count=0
        reg.register(0x1C, 0x00, page);
    }

    // ── 1Dh: Element Address Assignment ────────────────────────────────
    {
        let mut page = vec![0u8; 20];
        page[0] = 0x1D;
        page[1] = 0x12; // Page length (18)
        // MTE first address + count
        page[2] = (start_picker >> 8) as u8;
        page[3] = start_picker as u8;
        page[4] = (num_pickers >> 8) as u8;
        page[5] = num_pickers as u8;
        // STE first address + count
        page[6] = (start_slot >> 8) as u8;
        page[7] = start_slot as u8;
        page[8] = (num_slots >> 8) as u8;
        page[9] = num_slots as u8;
        // IEE first address + count
        page[10] = (start_iee >> 8) as u8;
        page[11] = start_iee as u8;
        page[12] = (num_iee >> 8) as u8;
        page[13] = num_iee as u8;
        // DTE first address + count
        page[14] = (start_drive >> 8) as u8;
        page[15] = start_drive as u8;
        page[16] = (num_drives >> 8) as u8;
        page[17] = num_drives as u8;
        reg.register(0x1D, 0x00, page);
    }

    // ── 1Eh: Transport Geometry Parameters ─────────────────────────────
    {
        let mut page = vec![0u8; 4];
        page[0] = 0x1E;
        page[1] = 0x02; // Page length (2)
        page[2] = 0x00; // Rotate=0 (no double-sided)
        page[3] = 0x00; // Member number=0 (single transport)
        reg.register(0x1E, 0x00, page);
    }

    // ── 1Fh/00h: Device Capabilities ───────────────────────────────────
    {
        let mut page = vec![0u8; 20];
        page[0] = 0x1F;
        page[1] = 0x12; // Page length (18)
        // Byte 2: Storage flags
        //   Bit 7: StorDT=1 (drives can store cartridges)
        //   Bit 6: StorIE=1 (I/E can store)
        //   Bit 5: StorST=1 (storage can store)
        //   Bit 4: StorMT=0 (accessor cannot store)
        //   Bit 2: VTRP=1 (volume tag reader present)
        //   Bit 1: S2C=1 (SMC-2 capabilities)
        page[2] = 0xE6; // 1110_0110 = StorDT|StorIE|StorST | VTRP|S2C
        // Byte 3: reserved
        // Bytes 4-19: Movement matrix
        // MT→X: byte 4 (MT as source)
        //   bit 7=MTE_to_DTE? no  bit 6=MTE_to_IE? no  bit 5=MTE_to_ST? no  bit 4=MTE_to_MT? no
        page[4] = 0x00; // MTE cannot be source for normal moves
        // DTE→X: byte 8
        //   bit 7=DTE_to_DTE? yes  bit 6=DTE_to_IE? yes  bit 5=DTE_to_ST? yes  bit 4=DTE_to_MT? no
        page[8] = 0xE0; // 1110_0000
        // STE→X: byte 12
        page[12] = 0xE0; // STE→DTE, STE→IE, STE→ST
        // IEE→X: byte 16
        page[16] = 0xE0; // IEE→DTE, IEE→IE, IEE→ST
        reg.register(0x1F, 0x00, page);
    }

    // ── 1Fh/41h: Extended Device Capabilities ──────────────────────────
    {
        // SPF=1 format: page_code, subpage, length(2), data...
        let mut page = vec![0u8; 20];
        page[0] = 0x5F; // Page code 1Fh with SPF=1 (bit 6 set)
        page[1] = 0x41; // Subpage code
        page[2] = 0x00; // Page length MSB
        page[3] = 0x10; // Page length LSB (16)
        // Byte 4: flags
        //   Bit 6: MVPRV=1 (prevent moves to I/E when medium removal prevented)
        //   Bit 3: USRCL=1 (user control I/E close)
        //   Bit 2: USROP=1 (user control I/E open)
        //   Bit 0: IEST=1 (detect medium in I/E)
        page[4] = 0x4D; // 0100_1101
        // Byte 5: flags
        //   Bit 3: IEMGZ=1 (I/E magazine)
        //   Bit 2: SMGZ=1 (storage magazine)
        page[5] = 0x0C; // 0000_1100
        // Byte 6: flags
        //   Bit 7: TREXC=1 (true exchange capable)
        //   Bit 6: LCKIE=1 (lock I/E with PREVENT/ALLOW)
        page[6] = 0xC0; // 1100_0000
        // Bytes 7-19: remaining flags (mostly 0 for our defaults)
        reg.register(0x1F, 0x41, page);
    }

    reg
}
