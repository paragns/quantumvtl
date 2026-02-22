//! Log page framework — trait, registry, and LOG SENSE/SELECT dispatch.
//!
//! Individual log page implementations register with the registry via the
//! LogPage trait. The framework handles header construction, page code
//! dispatch, and parameter formatting.

/// A single log parameter within a log page.
#[derive(Debug, Clone)]
pub struct LogParameter {
    /// Parameter code (16-bit).
    pub code: u16,
    /// Control byte: DU (bit 7), DS (bit 6), TSD (bit 5), ETC (bit 4),
    /// TMC (bits 3-2), FORMAT_AND_LINKING (bits 1-0).
    pub control: u8,
    /// Parameter value (variable length).
    pub value: Vec<u8>,
}

impl LogParameter {
    /// Create a counter parameter (binary format, 4 bytes).
    pub fn counter32(code: u16, value: u32) -> Self {
        Self {
            code,
            control: 0x03, // binary format, list parameter
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Create a counter parameter (binary format, 8 bytes).
    pub fn counter64(code: u16, value: u64) -> Self {
        Self {
            code,
            control: 0x03, // binary format, list parameter
            value: value.to_be_bytes().to_vec(),
        }
    }

    /// Serialize this parameter into the log page wire format.
    /// Format: code(2) + control(1) + length(1) + value(N).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.value.len());
        bytes.push(((self.code >> 8) & 0xFF) as u8);
        bytes.push((self.code & 0xFF) as u8);
        bytes.push(self.control);
        bytes.push(self.value.len() as u8);
        bytes.extend(&self.value);
        bytes
    }
}

/// Trait for a single log page implementation.
pub trait LogPage: Send + Sync {
    /// Log page code (e.g., 0x0C for Sequential Access Device).
    fn page_code(&self) -> u8;

    /// Subpage code (0 for pages without subpages).
    fn subpage_code(&self) -> u8 {
        0
    }

    /// Return all parameters for this log page.
    fn parameters(&self) -> Vec<LogParameter>;

    /// Reset clearable counters (LOG SELECT).
    fn reset(&self) {}
}

/// Registry of log pages with dispatch methods.
pub struct LogPageRegistry {
    pages: Vec<Box<dyn LogPage>>,
}

impl LogPageRegistry {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Register a log page implementation.
    pub fn register(&mut self, page: Box<dyn LogPage>) {
        self.pages.push(page);
    }

    /// Build LOG SENSE response for a specific page.
    /// Returns the full page (header + parameters), or None if not found.
    pub fn get_page(&self, page_code: u8, subpage: u8) -> Option<Vec<u8>> {
        let page = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)?;

        let params = page.parameters();
        let mut param_bytes = Vec::new();
        for p in &params {
            param_bytes.extend(p.to_bytes());
        }

        let page_length = param_bytes.len() as u16;

        // Log page header: page_code(1) + subpage(1) + page_length(2) + params
        let mut result = Vec::with_capacity(4 + param_bytes.len());
        let byte0 = page.page_code() & 0x3F;
        // If subpage != 0, set SPF bit
        let byte0 = if page.subpage_code() != 0 {
            byte0 | 0x40
        } else {
            byte0
        };
        result.push(byte0);
        result.push(page.subpage_code());
        result.push(((page_length >> 8) & 0xFF) as u8);
        result.push((page_length & 0xFF) as u8);
        result.extend(param_bytes);

        Some(result)
    }

    /// Build the supported log pages page (page 00h).
    pub fn supported_pages(&self) -> Vec<u8> {
        let mut page_codes: Vec<u8> = self.pages.iter().map(|p| p.page_code()).collect();
        page_codes.sort();
        page_codes.dedup();
        // Include page 00h itself
        if !page_codes.contains(&0x00) {
            page_codes.insert(0, 0x00);
        }

        let page_length = page_codes.len() as u16;
        let mut result = Vec::with_capacity(4 + page_codes.len());
        result.push(0x00); // page code
        result.push(0x00); // subpage
        result.push(((page_length >> 8) & 0xFF) as u8);
        result.push((page_length & 0xFF) as u8);
        result.extend(page_codes);
        result
    }

    /// Reset a specific page's counters (LOG SELECT).
    pub fn reset_page(&self, page_code: u8, subpage: u8) -> bool {
        if let Some(page) = self
            .pages
            .iter()
            .find(|p| p.page_code() == page_code && p.subpage_code() == subpage)
        {
            page.reset();
            true
        } else {
            false
        }
    }

    /// Reset all pages (LOG SELECT with page code 0).
    pub fn reset_all(&self) {
        for page in &self.pages {
            page.reset();
        }
    }
}

// --- Stub log pages ---

/// A stub log page that returns fixed/zero parameters.
pub struct StubLogPage {
    code: u8,
    params: Vec<LogParameter>,
}

impl StubLogPage {
    pub fn new(code: u8, params: Vec<LogParameter>) -> Self {
        Self { code, params }
    }

    pub fn empty(code: u8) -> Self {
        Self {
            code,
            params: Vec::new(),
        }
    }
}

impl LogPage for StubLogPage {
    fn page_code(&self) -> u8 {
        self.code
    }

    fn parameters(&self) -> Vec<LogParameter> {
        self.params.clone()
    }
}

/// Create the default log page registry with stub implementations for all
/// mandatory log pages. These will be replaced with real implementations
/// in Track C.
pub fn default_registry() -> LogPageRegistry {
    let mut reg = LogPageRegistry::new();

    // LP 02h: Write Error Counters
    reg.register(Box::new(StubLogPage::new(
        0x02,
        vec![
            LogParameter::counter32(0x0000, 0), // Errors corrected without delay
            LogParameter::counter32(0x0001, 0), // Errors corrected with delay
            LogParameter::counter32(0x0002, 0), // Total rewrites/rereads
            LogParameter::counter32(0x0003, 0), // Total errors corrected
            LogParameter::counter32(0x0005, 0), // Total uncorrected errors
            LogParameter::counter64(0x0006, 0), // Total bytes processed
        ],
    )));

    // LP 03h: Read Error Counters
    reg.register(Box::new(StubLogPage::new(
        0x03,
        vec![
            LogParameter::counter32(0x0000, 0),
            LogParameter::counter32(0x0001, 0),
            LogParameter::counter32(0x0002, 0),
            LogParameter::counter32(0x0003, 0),
            LogParameter::counter32(0x0005, 0),
            LogParameter::counter64(0x0006, 0),
        ],
    )));

    // LP 06h: Non-Medium Errors
    reg.register(Box::new(StubLogPage::new(
        0x06,
        vec![LogParameter::counter32(0x0000, 0)],
    )));

    // LP 0Ch: Sequential Access Device (THE capacity page)
    reg.register(Box::new(StubLogPage::new(
        0x0C,
        vec![
            LogParameter::counter64(0x0000, 0), // Total channel write bytes
            LogParameter::counter64(0x0001, 0), // Total device write bytes
            LogParameter::counter64(0x0002, 0), // Total device read bytes
            LogParameter::counter64(0x0003, 0), // Total channel read bytes
            LogParameter::counter32(0x0004, 0), // Native capacity BOP→EOD (MB)
            LogParameter::counter32(0x0005, 0), // Native capacity BOP→EW (MB)
            LogParameter::counter32(0x0006, 0), // Min native capacity EW→LEOP (MB)
            LogParameter::counter32(0x0007, 0), // Native capacity BOP→current (MB)
            LogParameter::counter32(0x0008, 0), // Max native capacity in buffer (MB)
            LogParameter::counter64(0x0100, 0), // Cleaning requested
            LogParameter::counter32(0x8000, 0), // MB processed since cleaning
            LogParameter::counter32(0x8001, 0), // Lifetime load cycles
            LogParameter::counter32(0x8002, 0), // Lifetime cleaning cycles
            LogParameter::counter32(0x8003, 0), // Lifetime power-on seconds
        ],
    )));

    // LP 11h: DT Device Status
    reg.register(Box::new(StubLogPage::empty(0x11)));

    // LP 14h: Device Statistics
    reg.register(Box::new(StubLogPage::new(
        0x14,
        vec![
            LogParameter::counter32(0x0000, 0), // Lifetime volume loads
            LogParameter::counter32(0x0001, 0), // Lifetime cleaning operations
            LogParameter::counter32(0x0002, 0), // Lifetime power-on hours
            LogParameter::counter32(0x0003, 0), // Lifetime medium motion hours
        ],
    )));

    // LP 17h: Volume Statistics
    reg.register(Box::new(StubLogPage::empty(0x17)));

    // LP 1Bh: Data Compression
    reg.register(Box::new(StubLogPage::new(
        0x1B,
        vec![
            LogParameter::counter64(0x0000, 0), // Read compression ratio x100
            LogParameter::counter64(0x0001, 0), // Write compression ratio x100
        ],
    )));

    // LP 2Eh: TapeAlerts
    reg.register(Box::new(StubLogPage::empty(0x2E)));

    // LP 30h: Tape Usage (legacy)
    reg.register(Box::new(StubLogPage::empty(0x30)));

    // LP 31h: Tape Capacity (legacy)
    reg.register(Box::new(StubLogPage::empty(0x31)));

    reg
}
