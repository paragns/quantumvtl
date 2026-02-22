use iscsi_target::{ScsiDevice, ScsiResult};
use tracing::trace;

// SCSI command opcodes
const INQUIRY: u8 = 0x12;
const TEST_UNIT_READY: u8 = 0x00;
const REQUEST_SENSE: u8 = 0x03;

/// A SCSI Media Changer device emulating a Quantum Scalar library.
pub struct MediaChanger {
    inquiry_data: Vec<u8>,
    serial: String,
    vendor: String,
    product: String,
}

impl MediaChanger {
    pub fn new(model: &str, serial: &str) -> Self {
        let vendor = "QUANTUM ";
        let product = format!("{:<16}", model);
        let product = &product[..16]; // truncate to 16 chars
        let revision = "0100";

        let mut inq = vec![0u8; 96];
        // Byte 0: Peripheral qualifier (0) | Peripheral device type (0x08 = Medium Changer)
        inq[0] = 0x08;
        // Byte 1: RMB=0
        inq[1] = 0x00;
        // Byte 2: Version (0x05 = SPC-3)
        inq[2] = 0x05;
        // Byte 3: Response data format (2) | HiSup=1 → 0x12
        inq[3] = 0x12;
        // Byte 4: Additional length (96 - 5 = 91)
        inq[4] = 91;
        // Byte 5: SCCS=0, ACC=0, TPGS=0, 3PC=0, Protect=0
        inq[5] = 0x00;
        // Byte 6: EncServ=0, VS=0, MultiP=0, Addr16=0
        inq[6] = 0x00;
        // Byte 7: WBus16=0, Sync=0, CmdQue=1 (bit 1)
        inq[7] = 0x02;
        // Bytes 8-15: Vendor identification
        inq[8..16].copy_from_slice(vendor.as_bytes());
        // Bytes 16-31: Product identification
        inq[16..32].copy_from_slice(product.as_bytes());
        // Bytes 32-35: Product revision level
        inq[32..36].copy_from_slice(revision.as_bytes());
        // Bytes 36-55: Vendor-specific (zeros)
        // Bytes 56-57: Reserved
        // Bytes 58-59: Version descriptor 1 — SAM-2 (0x0040)
        inq[58] = 0x00;
        inq[59] = 0x40;
        // Bytes 60-61: Version descriptor 2 — SAM-4 (0x0060)
        inq[60] = 0x00;
        inq[61] = 0x60;
        // Bytes 62-63: Version descriptor 3 — SPC-3 (0x0300)
        inq[62] = 0x03;
        inq[63] = 0x00;
        // Bytes 64-65: Version descriptor 4 — SMC-2 (0x0320)
        inq[64] = 0x03;
        inq[65] = 0x20;

        Self {
            inquiry_data: inq,
            serial: serial.to_string(),
            vendor: vendor.to_string(),
            product: product.to_string(),
        }
    }
}

impl ScsiDevice for MediaChanger {
    fn execute_command(&self, cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode, "SMC command");

        match opcode {
            INQUIRY => handle_inquiry(cdb, &self.inquiry_data, &self.vendor, &self.product, &self.serial),
            TEST_UNIT_READY => ScsiResult {
                status: 0x00,
                data_in: Vec::new(),
                sense: Vec::new(),
            },
            REQUEST_SENSE => handle_request_sense(cdb),
            _ => {
                trace!(opcode, "unsupported SMC command");
                ScsiResult {
                    status: 0x02, // CHECK CONDITION
                    data_in: Vec::new(),
                    sense: build_sense(0x05, 0x20, 0x00), // ILLEGAL REQUEST, INVALID COMMAND OPERATION CODE
                }
            }
        }
    }
}

fn handle_inquiry(cdb: &[u8], standard_inq: &[u8], vendor: &str, product: &str, serial: &str) -> ScsiResult {
    let evpd = cdb[1] & 0x01 != 0;
    let page_code = cdb[2];
    let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

    if !evpd {
        // Standard INQUIRY
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

    // VPD pages
    match page_code {
        // Supported VPD Pages
        0x00 => {
            let data = vec![
                0x08, // Peripheral qualifier | device type (medium changer)
                0x00, // Page code
                0x00, // Reserved
                0x03, // Page length
                0x00, // Supported page: 0x00
                0x80, // Supported page: 0x80
                0x83, // Supported page: 0x83
            ];
            let mut data = data;
            if data.len() > alloc_len {
                data.truncate(alloc_len);
            }
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        // Unit Serial Number
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
        // Device Identification
        0x83 => {
            let mut data = vec![0x08, 0x83, 0x00, 0x00]; // header, page length filled later

            // Designation descriptor 1: T10 vendor ID based
            let t10_id = format!("{}{}{}", vendor, product, serial);
            let t10_bytes = t10_id.as_bytes();
            data.push(0x02); // Protocol identifier=0, Code set=2 (ASCII)
            data.push(0x01); // PIV=0, Association=0 (LUN), Designator type=1 (T10 vendor ID)
            data.push(0x00); // Reserved
            data.push(t10_bytes.len() as u8); // Designator length
            data.extend_from_slice(t10_bytes);

            // Fill in page length
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
        _ => {
            // INVALID FIELD IN CDB
            ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense(0x05, 0x24, 0x00),
            }
        }
    }
}

fn handle_request_sense(cdb: &[u8]) -> ScsiResult {
    let alloc_len = cdb[4] as usize;
    let mut data = build_sense(0x00, 0x00, 0x00); // NO SENSE
    if data.len() > alloc_len {
        data.truncate(alloc_len);
    }
    ScsiResult {
        status: 0x00,
        data_in: data,
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
