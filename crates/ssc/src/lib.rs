use iscsi_target::{ScsiDevice, ScsiResult};
use tracing::trace;

const INQUIRY: u8 = 0x12;
const TEST_UNIT_READY: u8 = 0x00;
const REQUEST_SENSE: u8 = 0x03;

/// A stub SCSI Tape Drive device emulating an IBM Ultrium LTO drive.
pub struct TapeDrive {
    inquiry_data: Vec<u8>,
    serial: String,
}

impl TapeDrive {
    pub fn new(serial: &str) -> Self {
        let vendor = "IBM     ";
        let product = "ULTRIUM-TD9     ";
        let revision = "0100";

        let mut inq = vec![0u8; 96];
        // Byte 0: Peripheral qualifier (0) | Peripheral device type (0x01 = Sequential Access)
        inq[0] = 0x01;
        // Byte 1: RMB=1 (removable media)
        inq[1] = 0x80;
        // Byte 2: Version (0x05 = SPC-3)
        inq[2] = 0x05;
        // Byte 3: Response data format (2) | HiSup=1 → 0x12
        inq[3] = 0x12;
        // Byte 4: Additional length (96 - 5 = 91)
        inq[4] = 91;
        // Byte 7: CmdQue=1
        inq[7] = 0x02;
        // Bytes 8-15: Vendor identification
        inq[8..16].copy_from_slice(vendor.as_bytes());
        // Bytes 16-31: Product identification
        inq[16..32].copy_from_slice(product.as_bytes());
        // Bytes 32-35: Product revision level
        inq[32..36].copy_from_slice(revision.as_bytes());
        // Version descriptors: SAM-2, SPC-3, SSC-3
        inq[58] = 0x00;
        inq[59] = 0x40; // SAM-2
        inq[62] = 0x03;
        inq[63] = 0x00; // SPC-3
        inq[64] = 0x03;
        inq[65] = 0x40; // SSC-3

        Self {
            inquiry_data: inq,
            serial: serial.to_string(),
        }
    }
}

impl ScsiDevice for TapeDrive {
    fn execute_command(&self, cdb: &[u8], _data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode, "SSC command");

        match opcode {
            INQUIRY => {
                let evpd = cdb[1] & 0x01 != 0;
                let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

                if !evpd {
                    let mut data = self.inquiry_data.clone();
                    if data.len() > alloc_len {
                        data.truncate(alloc_len);
                    }
                    return ScsiResult {
                        status: 0x00,
                        data_in: data,
                        sense: Vec::new(),
                    };
                }

                let page_code = cdb[2];
                match page_code {
                    0x00 => {
                        let mut data = vec![0x01, 0x00, 0x00, 0x02, 0x00, 0x80];
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
                        let serial_bytes = self.serial.as_bytes();
                        let mut data = vec![0x01, 0x80, 0x00, serial_bytes.len() as u8];
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
                    _ => ScsiResult {
                        status: 0x02,
                        data_in: Vec::new(),
                        sense: build_sense(0x05, 0x24, 0x00),
                    },
                }
            }
            TEST_UNIT_READY => ScsiResult {
                status: 0x00,
                data_in: Vec::new(),
                sense: Vec::new(),
            },
            REQUEST_SENSE => {
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
            _ => {
                trace!(opcode, "unsupported SSC command");
                ScsiResult {
                    status: 0x02,
                    data_in: Vec::new(),
                    sense: build_sense(0x05, 0x20, 0x00),
                }
            }
        }
    }
}

fn build_sense(sense_key: u8, asc: u8, ascq: u8) -> Vec<u8> {
    let mut sense = vec![0u8; 18];
    sense[0] = 0x70;
    sense[2] = sense_key & 0x0F;
    sense[7] = 10;
    sense[12] = asc;
    sense[13] = ascq;
    sense
}
