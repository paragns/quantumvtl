use std::sync::{Arc, Mutex};

use iscsi_target::{MediaLoadNotify, ScsiDevice, ScsiResult};
use tracing::trace;

// SCSI command opcodes
const INQUIRY: u8 = 0x12;
const TEST_UNIT_READY: u8 = 0x00;
const REQUEST_SENSE: u8 = 0x03;
const INITIALIZE_ELEMENT_STATUS: u8 = 0x07;
const MODE_SENSE_6: u8 = 0x1A;
const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
const MODE_SENSE_10: u8 = 0x5A;
const MOVE_MEDIUM: u8 = 0xA5;
const READ_ELEMENT_STATUS: u8 = 0xB8;

// Element type codes
const ELEM_MTE: u8 = 1; // Medium Transport Element (robot arm)
const ELEM_STE: u8 = 2; // Storage Element (slot)
const ELEM_IEE: u8 = 3; // Import/Export Element (mailslot)
const ELEM_DTE: u8 = 4; // Data Transfer Element (drive)

#[derive(Debug, Clone)]
struct Element {
    element_type: u8,
    full: bool,
    barcode: Option<String>,
    source_element: u16,
}

#[derive(Debug)]
struct ChangerState {
    start_picker: u16,
    num_pickers: u16,
    start_drive: u16,
    num_drives: u16,
    start_slot: u16,
    num_slots: u16,
    start_iee: u16,
    num_iee: u16,
    elements: Vec<Element>,
}

/// A SCSI Media Changer device emulating a Quantum Scalar library.
pub struct MediaChanger {
    inquiry_data: Vec<u8>,
    serial: String,
    vendor: String,
    product: String,
    state: Mutex<ChangerState>,
    drives: Vec<Arc<dyn MediaLoadNotify>>,
}

impl MediaChanger {
    pub fn new(
        model: &str,
        serial: &str,
        num_drives: u16,
        num_slots: u16,
        media_barcodes: &[String],
        drives: Vec<Arc<dyn MediaLoadNotify>>,
    ) -> Self {
        let vendor = "QUANTUM ";
        let product = format!("{:<16}", model);
        let product = &product[..16];
        let revision = "0100";

        let mut inq = vec![0u8; 96];
        inq[0] = 0x08; // Medium Changer
        inq[1] = 0x00;
        inq[2] = 0x05; // SPC-3
        inq[3] = 0x12; // Response data format 2, HiSup=1
        inq[4] = 91;   // Additional length
        inq[5] = 0x00;
        inq[6] = 0x00;
        inq[7] = 0x02; // CmdQue=1
        inq[8..16].copy_from_slice(vendor.as_bytes());
        inq[16..32].copy_from_slice(product.as_bytes());
        inq[32..36].copy_from_slice(revision.as_bytes());
        // Version descriptors
        inq[58] = 0x00;
        inq[59] = 0x40; // SAM-2
        inq[60] = 0x00;
        inq[61] = 0x60; // SAM-4
        inq[62] = 0x03;
        inq[63] = 0x00; // SPC-3
        inq[64] = 0x03;
        inq[65] = 0x20; // SMC-2

        // Element address layout:
        //   0            = MTE (robot arm)
        //   1..D         = DTE (drives)
        //   D+1..D+S     = STE (slots)
        //   D+S+1        = IEE (mailslot)
        let start_picker: u16 = 0;
        let num_pickers: u16 = 1;
        let start_drive: u16 = 1;
        let start_slot: u16 = 1 + num_drives;
        let start_iee: u16 = 1 + num_drives + num_slots;
        let num_iee: u16 = 1;

        let total_elements = 1 + num_drives + num_slots + num_iee;
        let mut elements = Vec::with_capacity(total_elements as usize);

        // MTE at address 0
        elements.push(Element {
            element_type: ELEM_MTE,
            full: false,
            barcode: None,
            source_element: 0,
        });

        // DTEs
        for _ in 0..num_drives {
            elements.push(Element {
                element_type: ELEM_DTE,
                full: false,
                barcode: None,
                source_element: 0,
            });
        }

        // STEs — place media in first N slots
        for i in 0..num_slots {
            let barcode = media_barcodes.get(i as usize).cloned();
            let full = barcode.is_some();
            elements.push(Element {
                element_type: ELEM_STE,
                full,
                barcode,
                source_element: 0,
            });
        }

        // IEE
        elements.push(Element {
            element_type: ELEM_IEE,
            full: false,
            barcode: None,
            source_element: 0,
        });

        let state = ChangerState {
            start_picker,
            num_pickers,
            start_drive,
            num_drives,
            start_slot,
            num_slots,
            start_iee,
            num_iee,
            elements,
        };

        Self {
            inquiry_data: inq,
            serial: serial.to_string(),
            vendor: vendor.to_string(),
            product: product.to_string(),
            state: Mutex::new(state),
            drives,
        }
    }
}

impl ScsiDevice for MediaChanger {
    fn execute_command(&self, cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode, "SMC command");

        match opcode {
            INQUIRY => handle_inquiry(
                cdb,
                &self.inquiry_data,
                &self.vendor,
                &self.product,
                &self.serial,
            ),
            TEST_UNIT_READY => ScsiResult {
                status: 0x00,
                data_in: Vec::new(),
                sense: Vec::new(),
            },
            REQUEST_SENSE => handle_request_sense(cdb),
            INITIALIZE_ELEMENT_STATUS | PREVENT_ALLOW_MEDIUM_REMOVAL => ScsiResult {
                status: 0x00,
                data_in: Vec::new(),
                sense: Vec::new(),
            },
            MODE_SENSE_6 => handle_mode_sense_6(cdb, &self.state),
            MODE_SENSE_10 => handle_mode_sense_10(cdb, &self.state),
            MOVE_MEDIUM => handle_move_medium(cdb, &self.state, &self.drives),
            READ_ELEMENT_STATUS => handle_read_element_status(cdb, &self.state),
            _ => {
                trace!(opcode, "unsupported SMC command");
                ScsiResult {
                    status: 0x02,
                    data_in: Vec::new(),
                    sense: build_sense(0x05, 0x20, 0x00),
                }
            }
        }
    }
}

fn handle_inquiry(
    cdb: &[u8],
    standard_inq: &[u8],
    vendor: &str,
    product: &str,
    serial: &str,
) -> ScsiResult {
    let evpd = cdb[1] & 0x01 != 0;
    let page_code = cdb[2];
    let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

    if !evpd {
        let mut data = standard_inq.to_vec();
        if data.len() > alloc_len {
            data.truncate(alloc_len);
        }
        return ScsiResult {
            status: 0x00,
            data_in: data,
            sense: Vec::new(),
        };
    }

    match page_code {
        0x00 => {
            let mut data = vec![
                0x08, 0x00, 0x00, 0x03, // header: device type, page, reserved, length
                0x00, 0x80, 0x83, // supported pages
            ];
            if data.len() > alloc_len {
                data.truncate(alloc_len);
            }
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        0x80 => {
            let serial_bytes = serial.as_bytes();
            let mut data = vec![0x08, 0x80, 0x00, serial_bytes.len() as u8];
            data.extend_from_slice(serial_bytes);
            if data.len() > alloc_len {
                data.truncate(alloc_len);
            }
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        0x83 => {
            let mut data = vec![0x08, 0x83, 0x00, 0x00];
            let t10_id = format!("{}{}{}", vendor, product, serial);
            let t10_bytes = t10_id.as_bytes();
            data.push(0x02); // Code set=ASCII
            data.push(0x01); // Designator type=T10 vendor ID
            data.push(0x00);
            data.push(t10_bytes.len() as u8);
            data.extend_from_slice(t10_bytes);
            let page_len = (data.len() - 4) as u8;
            data[3] = page_len;
            if data.len() > alloc_len {
                data.truncate(alloc_len);
            }
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        _ => ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x24, 0x00),
        },
    }
}

fn handle_request_sense(cdb: &[u8]) -> ScsiResult {
    let alloc_len = cdb[4] as usize;
    let mut data = build_sense(0x00, 0x00, 0x00);
    if data.len() > alloc_len {
        data.truncate(alloc_len);
    }
    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// MODE SENSE(6) — returns Element Address Assignment page (0x1D)
fn handle_mode_sense_6(cdb: &[u8], state: &Mutex<ChangerState>) -> ScsiResult {
    let page_code = cdb[2] & 0x3F;
    let alloc_len = cdb[4] as usize;

    if page_code != 0x1D && page_code != 0x3F {
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x24, 0x00),
        };
    }

    let st = state.lock().unwrap();
    let page_data = build_element_address_page(&st);

    // MODE SENSE(6) header: 4 bytes
    let mut data = Vec::with_capacity(4 + page_data.len());
    let mode_data_len = (3 + page_data.len()) as u8; // everything after byte 0
    data.push(mode_data_len);
    data.push(0x00); // Medium type
    data.push(0x00); // Device-specific parameter
    data.push(0x00); // Block descriptor length
    data.extend_from_slice(&page_data);

    if data.len() > alloc_len {
        data.truncate(alloc_len);
    }

    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// MODE SENSE(10) — returns Element Address Assignment page (0x1D)
fn handle_mode_sense_10(cdb: &[u8], state: &Mutex<ChangerState>) -> ScsiResult {
    let page_code = cdb[2] & 0x3F;
    let alloc_len = ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    if page_code != 0x1D && page_code != 0x3F {
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x24, 0x00),
        };
    }

    let st = state.lock().unwrap();
    let page_data = build_element_address_page(&st);

    // MODE SENSE(10) header: 8 bytes
    let mut data = Vec::with_capacity(8 + page_data.len());
    let mode_data_len = (7 + page_data.len()) as u16; // everything after bytes 0-1
    data.push((mode_data_len >> 8) as u8);
    data.push(mode_data_len as u8);
    data.push(0x00); // Medium type
    data.push(0x00); // Device-specific parameter
    data.push(0x00); // Long LBA
    data.push(0x00); // Reserved
    data.push(0x00); // Block descriptor length (high)
    data.push(0x00); // Block descriptor length (low)
    data.extend_from_slice(&page_data);

    if data.len() > alloc_len {
        data.truncate(alloc_len);
    }

    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// Build Element Address Assignment mode page (0x1D)
fn build_element_address_page(st: &ChangerState) -> Vec<u8> {
    let mut page = vec![0u8; 20];
    page[0] = 0x1D; // Page code
    page[1] = 0x12; // Page length (18 bytes)
    // MTE first address + count
    page[2] = (st.start_picker >> 8) as u8;
    page[3] = st.start_picker as u8;
    page[4] = (st.num_pickers >> 8) as u8;
    page[5] = st.num_pickers as u8;
    // STE first address + count
    page[6] = (st.start_slot >> 8) as u8;
    page[7] = st.start_slot as u8;
    page[8] = (st.num_slots >> 8) as u8;
    page[9] = st.num_slots as u8;
    // IEE first address + count
    page[10] = (st.start_iee >> 8) as u8;
    page[11] = st.start_iee as u8;
    page[12] = (st.num_iee >> 8) as u8;
    page[13] = st.num_iee as u8;
    // DTE first address + count
    page[14] = (st.start_drive >> 8) as u8;
    page[15] = st.start_drive as u8;
    page[16] = (st.num_drives >> 8) as u8;
    page[17] = st.num_drives as u8;
    // Bytes 18-19: reserved (already 0)
    page
}

/// MOVE MEDIUM — CDB[2-3]=transport, [4-5]=source, [6-7]=dest
fn handle_move_medium(
    cdb: &[u8],
    state: &Mutex<ChangerState>,
    drives: &[Arc<dyn MediaLoadNotify>],
) -> ScsiResult {
    let _transport = ((cdb[2] as u16) << 8) | cdb[3] as u16;
    let source = ((cdb[4] as u16) << 8) | cdb[5] as u16;
    let dest = ((cdb[6] as u16) << 8) | cdb[7] as u16;
    let invert = cdb[10] & 0x01 != 0;

    if invert {
        // We don't support invert
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x24, 0x00), // INVALID FIELD IN CDB
        };
    }

    let mut st = state.lock().unwrap();
    let src_idx = source as usize;
    let dst_idx = dest as usize;

    if src_idx >= st.elements.len() || dst_idx >= st.elements.len() {
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x21, 0x01), // LOGICAL BLOCK ADDRESS OUT OF RANGE
        };
    }

    if !st.elements[src_idx].full {
        // Source element is empty
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x3B, 0x0D), // MEDIUM SOURCE ELEMENT EMPTY
        };
    }

    if st.elements[dst_idx].full {
        // Destination element is full
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x3B, 0x0E), // MEDIUM DESTINATION ELEMENT FULL
        };
    }

    // Notify source drive (if DTE) that media is being unloaded
    if st.elements[src_idx].element_type == ELEM_DTE {
        let drive_idx = src_idx - st.start_drive as usize;
        if let Some(drive) = drives.get(drive_idx) {
            drive.media_unloaded();
        }
    }

    // Transfer the media
    let barcode = st.elements[src_idx].barcode.take();
    st.elements[src_idx].full = false;

    st.elements[dst_idx].full = true;
    st.elements[dst_idx].barcode.clone_from(&barcode);
    st.elements[dst_idx].source_element = source;

    // Clear source's source_element
    st.elements[src_idx].source_element = 0;

    // Notify destination drive (if DTE) that media is loaded
    if st.elements[dst_idx].element_type == ELEM_DTE {
        let drive_idx = dst_idx - st.start_drive as usize;
        if let Some(drive) = drives.get(drive_idx) {
            if let Some(ref bc) = barcode {
                drive.media_loaded(bc);
            }
        }
    }

    trace!(source, dest, "MOVE MEDIUM complete");

    ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// READ ELEMENT STATUS — the big one. Returns element descriptors for mtx status.
fn handle_read_element_status(cdb: &[u8], state: &Mutex<ChangerState>) -> ScsiResult {
    let voltag = cdb[1] & 0x10 != 0;
    let type_filter = cdb[1] & 0x0F;
    let start_addr = ((cdb[2] as u16) << 8) | cdb[3] as u16;
    let num_elements = ((cdb[4] as u16) << 8) | cdb[5] as u16;
    let alloc_len =
        ((cdb[6] as usize) << 16) | ((cdb[7] as usize) << 8) | (cdb[8] as usize);

    let st = state.lock().unwrap();

    // Descriptor length: 12 base + 36 if voltag
    let desc_len: u16 = if voltag { 48 } else { 12 };

    // Determine which element types to report
    let types_to_report: Vec<u8> = if type_filter == 0 {
        vec![ELEM_MTE, ELEM_STE, ELEM_IEE, ELEM_DTE]
    } else {
        vec![type_filter]
    };

    // Build per-type pages
    let mut report_data = Vec::new();
    let mut total_elements_reported: u16 = 0;
    let end_addr = start_addr.saturating_add(num_elements);

    for &etype in &types_to_report {
        // Collect elements of this type within the requested range
        let mut descriptors = Vec::new();
        for (addr, elem) in st.elements.iter().enumerate() {
            let addr = addr as u16;
            if elem.element_type != etype {
                continue;
            }
            if addr < start_addr || addr >= end_addr {
                continue;
            }
            descriptors.push((addr, elem));
        }

        if descriptors.is_empty() {
            continue;
        }

        // Page header (8 bytes)
        let num_desc = descriptors.len() as u16;
        let desc_bytes = num_desc as u32 * desc_len as u32;

        let mut page_header = vec![0u8; 8];
        page_header[0] = etype;
        page_header[1] = if voltag { 0x80 } else { 0x00 }; // PVolTag
        page_header[2] = (desc_len >> 8) as u8;
        page_header[3] = desc_len as u8;
        // Reserved byte [4]
        // Total bytes of descriptors (BE24)
        page_header[5] = ((desc_bytes >> 16) & 0xFF) as u8;
        page_header[6] = ((desc_bytes >> 8) & 0xFF) as u8;
        page_header[7] = (desc_bytes & 0xFF) as u8;

        report_data.extend_from_slice(&page_header);

        for (addr, elem) in &descriptors {
            let mut desc = vec![0u8; desc_len as usize];
            // Element address
            desc[0] = (*addr >> 8) as u8;
            desc[1] = *addr as u8;
            // Flags
            let mut flags: u8 = 0x00;
            if elem.full {
                flags |= 0x01; // FULL bit
            }
            // ACCESS bit — always set for STE/IEE/MTE, set for DTE
            desc[2] = flags;
            // Byte 3: reserved
            // Byte 4-5: ASC/ASCQ = 0/0
            // Byte 6-7: reserved (byte 7 = SCSI ID for DTE)
            // Byte 8: reserved
            // Byte 9: SValid | medium_type
            let mut byte9: u8 = 0x00;
            if elem.full && elem.source_element != 0 {
                byte9 |= 0x80; // SValid
            }
            desc[9] = byte9;
            // Byte 10-11: source element address
            desc[10] = (elem.source_element >> 8) as u8;
            desc[11] = elem.source_element as u8;

            if voltag {
                // Volume tag: bytes 12-43 (32 bytes barcode, 4 reserved)
                if let Some(ref bc) = elem.barcode {
                    let bc_bytes = bc.as_bytes();
                    let copy_len = bc_bytes.len().min(32);
                    desc[12..12 + copy_len].copy_from_slice(&bc_bytes[..copy_len]);
                    // Pad remaining with spaces
                    for b in &mut desc[12 + copy_len..44] {
                        *b = b' ';
                    }
                } else {
                    // Empty: all spaces
                    for b in &mut desc[12..44] {
                        *b = b' ';
                    }
                }
                // Bytes 44-47: reserved (already 0)
            }

            report_data.extend_from_slice(&desc);
            total_elements_reported += 1;
        }
    }

    // Build the 8-byte main header
    let byte_count = report_data.len() as u32;
    let mut response = Vec::with_capacity(8 + report_data.len());
    // First element address
    response.push((start_addr >> 8) as u8);
    response.push(start_addr as u8);
    // Number of elements reported
    response.push((total_elements_reported >> 8) as u8);
    response.push(total_elements_reported as u8);
    // Reserved
    response.push(0x00);
    // Byte count of report data (BE24)
    response.push(((byte_count >> 16) & 0xFF) as u8);
    response.push(((byte_count >> 8) & 0xFF) as u8);
    response.push((byte_count & 0xFF) as u8);

    response.extend_from_slice(&report_data);

    if response.len() > alloc_len {
        response.truncate(alloc_len);
    }

    ScsiResult {
        status: 0x00,
        data_in: response,
        sense: Vec::new(),
    }
}

fn build_sense(sense_key: u8, asc: u8, ascq: u8) -> Vec<u8> {
    let mut sense = vec![0u8; 18];
    sense[0] = 0x70; // Response code: current, fixed format
    sense[2] = sense_key & 0x0F;
    sense[7] = 10; // Additional sense length
    sense[12] = asc;
    sense[13] = ascq;
    sense
}
