//! Mode page framework — trait, registry, and MODE SENSE/SELECT dispatch.
//!
//! Individual mode page implementations register with the registry via the
//! ModePage trait. The framework handles header construction, page code
//! dispatch, and PC (page control) field semantics.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Page Control field values from MODE SENSE CDB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageControl {
    Current = 0,
    Changeable = 1,
    Default = 2,
    Saved = 3,
}

impl PageControl {
    pub fn from_byte(b: u8) -> Self {
        match (b >> 6) & 0x03 {
            0 => Self::Current,
            1 => Self::Changeable,
            2 => Self::Default,
            3 => Self::Saved,
            _ => unreachable!(),
        }
    }
}

/// Trait for a single mode page implementation.
///
/// Each mode page provides its data for the four page control variants,
/// and can accept MODE SELECT updates to its current values.
pub trait ModePage: Send + Sync {
    /// Page code (e.g., 0x0F for Data Compression).
    fn page_code(&self) -> u8;

    /// Subpage code (0 for pages without subpages).
    fn subpage_code(&self) -> u8 {
        0
    }

    /// Page data for the given page control variant.
    /// Returns the page bytes WITHOUT the page header — the registry adds that.
    fn page_data(&self, pc: PageControl) -> Vec<u8>;

    /// Page length (data bytes, not including header).
    fn page_length(&self) -> u8;

    /// Whether this page is saveable (PS bit in MODE SENSE).
    fn saveable(&self) -> bool {
        false
    }

    /// Apply MODE SELECT data. The `data` slice is the page data WITHOUT
    /// the page header. Returns Ok(()) on success or an error description.
    fn apply_select(&self, _data: &[u8]) -> Result<(), &'static str> {
        Err("page not changeable")
    }
}

/// Registry of mode pages with dispatch methods.
#[derive(Default)]
pub struct ModePageRegistry {
    pages: Vec<Box<dyn ModePage>>,
}

impl ModePageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a mode page implementation.
    pub fn register(&mut self, page: Box<dyn ModePage>) {
        self.pages.push(page);
    }

    /// Build MODE SENSE response data for a specific page.
    /// Returns the page header + page data, or None if page not found.
    pub fn get_page(&self, page_code: u8, subpage: u8, pc: PageControl) -> Option<Vec<u8>> {
        let page = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)?;

        Some(self.build_page_response(page.as_ref(), pc))
    }

    /// Build MODE SENSE response for all pages (page code 0x3F).
    pub fn get_all_pages(&self, pc: PageControl) -> Vec<u8> {
        let mut result = Vec::new();
        for page in &self.pages {
            result.extend(self.build_page_response(page.as_ref(), pc));
        }
        result
    }

    /// Apply MODE SELECT to a specific page.
    pub fn apply_select(
        &self,
        page_code: u8,
        subpage: u8,
        data: &[u8],
    ) -> Result<(), &'static str> {
        let page = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)
            .ok_or("page not found")?;

        page.apply_select(data)
    }

    fn build_page_response(&self, page: &dyn ModePage, pc: PageControl) -> Vec<u8> {
        let data = page.page_data(pc);
        let has_subpage = page.subpage_code() != 0;

        if has_subpage {
            // Subpage format: page_code | SPF, subpage, length(16-bit)
            let total_len = data.len() as u16;
            let mut result = Vec::with_capacity(4 + data.len());
            let mut byte0 = page.page_code() & 0x3F;
            if page.saveable() && pc == PageControl::Current {
                byte0 |= 0x80; // PS bit
            }
            byte0 |= 0x40; // SPF bit (subpage format)
            result.push(byte0);
            result.push(page.subpage_code());
            result.push(((total_len >> 8) & 0xFF) as u8);
            result.push((total_len & 0xFF) as u8);
            result.extend(&data);
            result
        } else {
            // Standard format: page_code, length(8-bit)
            let mut result = Vec::with_capacity(2 + data.len());
            let mut byte0 = page.page_code() & 0x3F;
            if page.saveable() && pc == PageControl::Current {
                byte0 |= 0x80; // PS bit
            }
            result.push(byte0);
            result.push(page.page_length());
            result.extend(&data);
            result
        }
    }
}

// --- Stub mode pages that return hardcoded defaults ---

/// A simple stub mode page that returns fixed data for all page control variants.
pub struct StubModePage {
    code: u8,
    subpage: u8,
    data: Vec<u8>,
}

impl StubModePage {
    pub fn new(code: u8, data: Vec<u8>) -> Self {
        Self {
            code,
            subpage: 0,
            data,
        }
    }

    pub fn with_subpage(code: u8, subpage: u8, data: Vec<u8>) -> Self {
        Self {
            code,
            subpage,
            data,
        }
    }
}

impl ModePage for StubModePage {
    fn page_code(&self) -> u8 {
        self.code
    }

    fn subpage_code(&self) -> u8 {
        self.subpage
    }

    fn page_data(&self, pc: PageControl) -> Vec<u8> {
        match pc {
            PageControl::Changeable => vec![0x00; self.data.len()], // nothing changeable
            _ => self.data.clone(),
        }
    }

    fn page_length(&self) -> u8 {
        self.data.len() as u8
    }
}

// --- Data Compression mode page (0Fh) ---

/// Real Data Compression mode page backed by an `Arc<AtomicBool>` for DCE.
pub struct DataCompressionModePage {
    dce: Arc<AtomicBool>,
}

impl DataCompressionModePage {
    pub fn new(dce: Arc<AtomicBool>) -> Self {
        Self { dce }
    }

    fn build_data(&self, dce_on: bool) -> Vec<u8> {
        let mut data = vec![0u8; 14];
        // Byte 0: DCC=1 (bit 6, device capable), DCE=dce_on (bit 7)
        data[0] = 0x40; // DCC=1
        if dce_on {
            data[0] |= 0x80; // DCE=1
        }
        // Bytes 2-5: Compression algorithm (00 00 00 FF = unregistered/default)
        data[2] = 0x00;
        data[3] = 0x00;
        data[4] = 0x00;
        data[5] = 0xFF;
        // Bytes 6-9: Decompression algorithm (00 00 00 FF)
        data[6] = 0x00;
        data[7] = 0x00;
        data[8] = 0x00;
        data[9] = 0xFF;
        data
    }
}

impl ModePage for DataCompressionModePage {
    fn page_code(&self) -> u8 {
        0x0F
    }

    fn page_data(&self, pc: PageControl) -> Vec<u8> {
        match pc {
            PageControl::Current | PageControl::Saved => {
                self.build_data(self.dce.load(Ordering::Relaxed))
            }
            PageControl::Default => self.build_data(true), // default is compression on
            PageControl::Changeable => {
                let mut data = vec![0u8; 14];
                data[0] = 0x80; // only DCE bit is changeable
                data
            }
        }
    }

    fn page_length(&self) -> u8 {
        14
    }

    fn saveable(&self) -> bool {
        true
    }

    fn apply_select(&self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            return Err("no data for compression page");
        }
        let dce = data[0] & 0x80 != 0;
        self.dce.store(dce, Ordering::Relaxed);
        Ok(())
    }
}

// --- Medium Partition mode page (11h) ---

/// Shared partition state between Mode Page 11h and FORMAT MEDIUM.
pub struct PartitionPageState {
    /// Maximum additional partitions (geometry.max_partitions - 1, read-only).
    pub max_additional: u8,
    /// Current additional partitions (media.partitions.len() - 1).
    pub current_additional: u8,
    /// Set by MODE SELECT, consumed by FORMAT MEDIUM.
    pub pending_additional: Option<u8>,
}

/// Thread-safe shared partition state.
pub type SharedPartitionState = Arc<Mutex<PartitionPageState>>;

/// Real Medium Partition mode page (11h) backed by shared partition state.
///
/// SSC-4 / LTO spec: page code 0x11, variable length.
/// Byte 0: max additional partitions (read-only)
/// Byte 1: additional partitions defined
/// Byte 2: FDP | SDP | IDP | PSUM(2) | POFM | CLEAR | ADDP
/// Byte 3: medium format recognition (0x03 = format and partition recognition)
/// Bytes 4+: partition size descriptors (2 bytes each)
pub struct MediumPartitionModePage {
    state: SharedPartitionState,
}

impl MediumPartitionModePage {
    pub fn new(state: SharedPartitionState) -> Self {
        Self { state }
    }
}

impl ModePage for MediumPartitionModePage {
    fn page_code(&self) -> u8 {
        0x11
    }

    fn page_data(&self, pc: PageControl) -> Vec<u8> {
        let st = self.state.lock().unwrap();
        match pc {
            PageControl::Current | PageControl::Saved => {
                // SSC spec: 6 fixed bytes (bytes 2-7) + 2 bytes per partition size descriptor.
                // Bytes: [max_add, add_def, flags, MFR, reserved, reserved, size0_hi, size0_lo, ...]
                let additional = st.current_additional;
                let total_partitions = additional as usize + 1;
                let mut data = vec![0u8; 6 + total_partitions * 2];
                data[0] = st.max_additional; // max additional partitions
                data[1] = additional; // additional partitions defined
                // Byte 2: IDP=1 if partitioned (bit 5), PSUM=00 (units in MB), rest 0
                if additional > 0 {
                    data[2] = 0x20; // IDP=1
                }
                data[3] = 0x03; // medium format recognition
                // Bytes 4-5: reserved (already zero)
                // Bytes 6+: partition size descriptors (all zeros = capacity split evenly by drive)
                data
            }
            PageControl::Default => {
                // Default: single partition (additional=0), 6 fixed + 2 for one size descriptor
                let mut data = vec![0u8; 8];
                data[0] = st.max_additional;
                data[1] = 0; // additional=0
                data[2] = 0;
                data[3] = 0x03;
                // Bytes 4-5: reserved
                // Bytes 6-7: partition 0 size = 0
                data
            }
            PageControl::Changeable => {
                // Changeable mask: bytes 1, 2 are changeable; sizes are changeable
                let additional = st.current_additional;
                let total_partitions = additional as usize + 1;
                let mut data = vec![0u8; 6 + total_partitions * 2];
                data[0] = 0x00; // max_additional not changeable
                data[1] = 0xFF; // additional partitions changeable
                data[2] = 0x78; // IDP + SDP + FDP + PSUM bits changeable
                data[3] = 0x00; // MFR not changeable
                // Bytes 4-5: reserved, not changeable
                // Partition sizes changeable
                for i in 0..total_partitions * 2 {
                    data[6 + i] = 0xFF;
                }
                data
            }
        }
    }

    fn page_length(&self) -> u8 {
        let st = self.state.lock().unwrap();
        let total_partitions = st.current_additional as usize + 1;
        (6 + total_partitions * 2) as u8
    }

    fn saveable(&self) -> bool {
        true
    }

    fn apply_select(&self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() < 2 {
            return Err("mode page 11h data too short");
        }
        let mut st = self.state.lock().unwrap();
        let additional = data[1];
        if additional > st.max_additional {
            return Err("additional partitions exceeds maximum");
        }
        // IDP (bit 5 of byte 2) must be set if additional > 0
        if additional > 0 && data.len() >= 3 && data[2] & 0x20 == 0 {
            return Err("IDP must be set when defining additional partitions");
        }
        st.pending_additional = Some(additional);
        Ok(())
    }
}

/// Create the default mode page registry with stub implementations for all
/// mandatory mode pages, plus the real Data Compression mode page (0Fh)
/// and Medium Partition mode page (11h).
pub fn default_registry(
    dce: Arc<AtomicBool>,
    partition_state: SharedPartitionState,
) -> ModePageRegistry {
    let mut reg = ModePageRegistry::new();

    // MP 01h: Read-Write Error Recovery (12 bytes)
    reg.register(Box::new(StubModePage::new(0x01, vec![0; 10])));

    // MP 02h: Disconnect-Reconnect (14 bytes)
    reg.register(Box::new(StubModePage::new(0x02, vec![0; 14])));

    // MP 0Ah: Control (10 bytes)
    reg.register(Box::new(StubModePage::new(0x0A, vec![0; 10])));

    // MP 0Ah[01h]: Control Extension (28 bytes)
    reg.register(Box::new(StubModePage::with_subpage(
        0x0A,
        0x01,
        vec![0; 28],
    )));

    // MP 0Ah[F0h]: Control Data Protection (4 bytes)
    reg.register(Box::new(StubModePage::with_subpage(0x0A, 0xF0, vec![0; 4])));

    // MP 0Fh: Data Compression — real implementation with shared DCE flag
    reg.register(Box::new(DataCompressionModePage::new(dce)));

    // MP 10h: Device Configuration (14 bytes)
    let mut dev_config = vec![0u8; 14];
    // Write delay time = 30 seconds = 0x012C (in 100ms units)
    dev_config[4] = 0x01;
    dev_config[5] = 0x2C;
    // SELECT DATA COMPRESSION ALGORITHM = 01h (default)
    dev_config[12] = 0x01;
    reg.register(Box::new(StubModePage::new(0x10, dev_config)));

    // MP 10h[01h]: Device Configuration Extension (28 bytes)
    let mut dev_config_ext = vec![0u8; 28];
    // WRITE MODE = 00h (overwrite allowed)
    // SHORT ERASE MODE = 02h
    dev_config_ext[1] = 0x02;
    reg.register(Box::new(StubModePage::with_subpage(
        0x10,
        0x01,
        dev_config_ext,
    )));

    // MP 11h: Medium Partition — real implementation with shared partition state
    reg.register(Box::new(MediumPartitionModePage::new(partition_state)));

    // MP 1Ah: Power Condition (10 bytes)
    reg.register(Box::new(StubModePage::new(0x1A, vec![0; 10])));

    // MP 1Ch: Informational Exceptions Control (10 bytes)
    reg.register(Box::new(StubModePage::new(0x1C, vec![0; 10])));

    // MP 1Dh: Medium Configuration (28 bytes)
    reg.register(Box::new(StubModePage::new(0x1D, vec![0; 28])));

    reg
}
