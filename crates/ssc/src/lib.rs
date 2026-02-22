use std::sync::Mutex;

use iscsi_target::{MediaLoadNotify, ScsiDevice, ScsiResult};
use tracing::trace;

// SCSI command opcodes
const TEST_UNIT_READY: u8 = 0x00;
const REWIND: u8 = 0x01;
const REQUEST_SENSE: u8 = 0x03;
const READ_BLOCK_LIMITS: u8 = 0x05;
const READ_6: u8 = 0x08;
const WRITE_6: u8 = 0x0A;
const WRITE_FILEMARKS: u8 = 0x10;
const SPACE_6: u8 = 0x11;
const INQUIRY: u8 = 0x12;
const MODE_SENSE_6: u8 = 0x1A;
const LOAD_UNLOAD: u8 = 0x1B;
const READ_POSITION: u8 = 0x34;

const MAX_BLOCK_SIZE: u32 = 256 * 1024; // 256 KB

enum TapeRecord {
    Data(Vec<u8>),
    Filemark,
}

struct TapeMedia {
    _barcode: String,
    records: Vec<TapeRecord>,
    position: usize, // 0 = BOT, len = EOD
}

struct DriveState {
    media: Option<TapeMedia>,
}

/// A SCSI Tape Drive device emulating an IBM Ultrium LTO drive.
pub struct TapeDrive {
    inquiry_data: Vec<u8>,
    serial: String,
    state: Mutex<DriveState>,
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
            state: Mutex::new(DriveState { media: None }),
        }
    }
}

impl MediaLoadNotify for TapeDrive {
    fn media_loaded(&self, barcode: &str) {
        let mut st = self.state.lock().unwrap();
        trace!(barcode, "tape media loaded into drive");
        st.media = Some(TapeMedia {
            _barcode: barcode.to_string(),
            records: Vec::new(),
            position: 0,
        });
    }

    fn media_unloaded(&self) {
        let mut st = self.state.lock().unwrap();
        trace!("tape media unloaded from drive");
        st.media = None;
    }
}

impl ScsiDevice for TapeDrive {
    fn execute_command(&self, cdb: &[u8], data_out: &[u8]) -> ScsiResult {
        let opcode = cdb[0];
        trace!(opcode, "SSC command");

        match opcode {
            INQUIRY => handle_inquiry(cdb, &self.inquiry_data, &self.serial),
            TEST_UNIT_READY => {
                let st = self.state.lock().unwrap();
                if st.media.is_none() {
                    return ScsiResult {
                        status: 0x02,
                        data_in: Vec::new(),
                        sense: build_sense_no_media(),
                    };
                }
                ScsiResult {
                    status: 0x00,
                    data_in: Vec::new(),
                    sense: Vec::new(),
                }
            }
            REQUEST_SENSE => handle_request_sense(cdb),
            REWIND => {
                let mut st = self.state.lock().unwrap();
                match st.media.as_mut() {
                    Some(media) => {
                        media.position = 0;
                        trace!("REWIND: position=0");
                        ScsiResult {
                            status: 0x00,
                            data_in: Vec::new(),
                            sense: Vec::new(),
                        }
                    }
                    None => ScsiResult {
                        status: 0x02,
                        data_in: Vec::new(),
                        sense: build_sense_no_media(),
                    },
                }
            }
            READ_6 => handle_read_6(cdb, &self.state),
            WRITE_6 => handle_write_6(cdb, data_out, &self.state),
            WRITE_FILEMARKS => handle_write_filemarks(cdb, &self.state),
            SPACE_6 => handle_space_6(cdb, &self.state),
            READ_POSITION => handle_read_position(&self.state),
            READ_BLOCK_LIMITS => handle_read_block_limits(),
            MODE_SENSE_6 => handle_mode_sense_6(cdb, &self.state),
            LOAD_UNLOAD => {
                // No-op — the changer handles load/unload via MediaLoadNotify
                trace!("LOAD/UNLOAD: no-op");
                ScsiResult {
                    status: 0x00,
                    data_in: Vec::new(),
                    sense: Vec::new(),
                }
            }
            _ => {
                trace!(opcode, "unsupported SSC command");
                ScsiResult {
                    status: 0x02,
                    data_in: Vec::new(),
                    sense: build_sense(0x05, 0x20, 0x00), // INVALID COMMAND OPERATION CODE
                }
            }
        }
    }
}

fn handle_inquiry(cdb: &[u8], standard_inq: &[u8], serial: &str) -> ScsiResult {
    let evpd = cdb[1] & 0x01 != 0;
    let alloc_len = ((cdb[3] as usize) << 8) | (cdb[4] as usize);

    if !evpd {
        let mut data = standard_inq.to_vec();
        data.truncate(alloc_len);
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
            data.truncate(alloc_len);
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        0x80 => {
            let serial_bytes = serial.as_bytes();
            let mut data = vec![0x01, 0x80, 0x00, serial_bytes.len() as u8];
            data.extend_from_slice(serial_bytes);
            data.truncate(alloc_len);
            ScsiResult {
                status: 0x00,
                data_in: data,
                sense: Vec::new(),
            }
        }
        _ => ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x05, 0x24, 0x00), // INVALID FIELD IN CDB
        },
    }
}

fn handle_request_sense(cdb: &[u8]) -> ScsiResult {
    let alloc_len = cdb[4] as usize;
    let mut data = build_sense(0x00, 0x00, 0x00);
    data.truncate(alloc_len);
    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// READ(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length
fn handle_read_6(cdb: &[u8], state: &Mutex<DriveState>) -> ScsiResult {
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    let mut st = state.lock().unwrap();
    let media = match st.media.as_mut() {
        Some(m) => m,
        None => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense_no_media(),
            }
        }
    };

    if transfer_length == 0 {
        return ScsiResult {
            status: 0x00,
            data_in: Vec::new(),
            sense: Vec::new(),
        };
    }

    // Check for EOD (blank check)
    if media.position >= media.records.len() {
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense: build_sense(0x08, 0x00, 0x05), // BLANK CHECK: END-OF-DATA
        };
    }

    // Check for filemark
    if matches!(media.records[media.position], TapeRecord::Filemark) {
        media.position += 1;
        let mut sense = build_sense(0x00, 0x00, 0x01); // NO SENSE: FILEMARK DETECTED
        sense[2] |= 0x80; // Set FILEMARK bit in byte 2
        return ScsiResult {
            status: 0x02,
            data_in: Vec::new(),
            sense,
        };
    }

    if fixed {
        // Fixed-block mode: return transfer_length blocks
        // For simplicity, use the first block's size as the block size
        let block_size = match &media.records[media.position] {
            TapeRecord::Data(d) => d.len(),
            TapeRecord::Filemark => unreachable!(),
        };
        let mut data = Vec::with_capacity(block_size * transfer_length as usize);
        let mut blocks_read: u32 = 0;

        for _ in 0..transfer_length {
            if media.position >= media.records.len() {
                // Underrun: return what we have with residual info
                break;
            }
            match &media.records[media.position] {
                TapeRecord::Data(d) => {
                    data.extend_from_slice(d);
                    media.position += 1;
                    blocks_read += 1;
                }
                TapeRecord::Filemark => {
                    break;
                }
            }
        }

        if blocks_read < transfer_length {
            // Partial read — report residual in sense info field
            let residual = transfer_length - blocks_read;
            let mut sense = build_sense(0x00, 0x00, 0x01);
            sense[2] |= 0x80; // FILEMARK bit
            // Sense info field (bytes 3-6) = residual count
            sense[3] = ((residual >> 24) & 0xFF) as u8;
            sense[4] = ((residual >> 16) & 0xFF) as u8;
            sense[5] = ((residual >> 8) & 0xFF) as u8;
            sense[6] = (residual & 0xFF) as u8;
            return ScsiResult {
                status: 0x02,
                data_in: data,
                sense,
            };
        }

        ScsiResult {
            status: 0x00,
            data_in: data,
            sense: Vec::new(),
        }
    } else {
        // Variable-block mode: return one block, up to transfer_length bytes
        let data = match &media.records[media.position] {
            TapeRecord::Data(d) => d.clone(),
            TapeRecord::Filemark => unreachable!(),
        };
        media.position += 1;

        let mut result_data = data;
        if result_data.len() > transfer_length as usize {
            result_data.truncate(transfer_length as usize);
        }

        ScsiResult {
            status: 0x00,
            data_in: result_data,
            sense: Vec::new(),
        }
    }
}

/// WRITE(6) — CDB[1] bit 0=FIXED, CDB[2-4]=transfer length
fn handle_write_6(cdb: &[u8], data_out: &[u8], state: &Mutex<DriveState>) -> ScsiResult {
    let fixed = cdb[1] & 0x01 != 0;
    let transfer_length =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    let mut st = state.lock().unwrap();
    let media = match st.media.as_mut() {
        Some(m) => m,
        None => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense_no_media(),
            }
        }
    };

    if transfer_length == 0 {
        return ScsiResult {
            status: 0x00,
            data_in: Vec::new(),
            sense: Vec::new(),
        };
    }

    // Truncate from current position (overwrite mode)
    media.records.truncate(media.position);

    if fixed {
        // Fixed-block mode: data_out contains transfer_length blocks
        // Block size is data_out.len() / transfer_length
        if transfer_length > 0 {
            let block_size = data_out.len() / transfer_length as usize;
            for i in 0..transfer_length as usize {
                let start = i * block_size;
                let end = start + block_size;
                let block = data_out[start..end].to_vec();
                media.records.push(TapeRecord::Data(block));
                media.position += 1;
            }
        }
    } else {
        // Variable-block mode: data_out is one block
        media.records.push(TapeRecord::Data(data_out.to_vec()));
        media.position += 1;
    }

    trace!(
        position = media.position,
        total_records = media.records.len(),
        "WRITE complete"
    );

    ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// WRITE FILEMARKS — CDB[2-4]=count
fn handle_write_filemarks(cdb: &[u8], state: &Mutex<DriveState>) -> ScsiResult {
    let count = ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);

    let mut st = state.lock().unwrap();
    let media = match st.media.as_mut() {
        Some(m) => m,
        None => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense_no_media(),
            }
        }
    };

    // Truncate from current position
    media.records.truncate(media.position);

    for _ in 0..count {
        media.records.push(TapeRecord::Filemark);
        media.position += 1;
    }

    trace!(count, position = media.position, "WRITE FILEMARKS complete");

    ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// SPACE(6) — CDB[1] bits 0-2=code, CDB[2-4]=signed 24-bit count
fn handle_space_6(cdb: &[u8], state: &Mutex<DriveState>) -> ScsiResult {
    let code = cdb[1] & 0x07;
    // Signed 24-bit count
    let raw_count =
        ((cdb[2] as u32) << 16) | ((cdb[3] as u32) << 8) | (cdb[4] as u32);
    let count: i32 = if raw_count & 0x800000 != 0 {
        (raw_count | 0xFF000000) as i32
    } else {
        raw_count as i32
    };

    let mut st = state.lock().unwrap();
    let media = match st.media.as_mut() {
        Some(m) => m,
        None => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense_no_media(),
            }
        }
    };

    match code {
        0 => {
            // Space blocks
            if count >= 0 {
                for _ in 0..count {
                    if media.position >= media.records.len() {
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense: build_sense(0x08, 0x00, 0x05), // BLANK CHECK
                        };
                    }
                    if matches!(media.records[media.position], TapeRecord::Filemark) {
                        media.position += 1;
                        let mut sense = build_sense(0x00, 0x00, 0x01);
                        sense[2] |= 0x80; // FILEMARK bit
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense,
                        };
                    }
                    media.position += 1;
                }
            } else {
                for _ in 0..(-count) {
                    if media.position == 0 {
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense: build_sense(0x00, 0x00, 0x04), // BOM
                        };
                    }
                    media.position -= 1;
                    if matches!(media.records[media.position], TapeRecord::Filemark) {
                        let mut sense = build_sense(0x00, 0x00, 0x01);
                        sense[2] |= 0x80; // FILEMARK bit
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense,
                        };
                    }
                }
            }
        }
        1 => {
            // Space filemarks
            if count >= 0 {
                let mut fm_seen = 0i32;
                while fm_seen < count {
                    if media.position >= media.records.len() {
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense: build_sense(0x08, 0x00, 0x05), // BLANK CHECK
                        };
                    }
                    if matches!(media.records[media.position], TapeRecord::Filemark) {
                        fm_seen += 1;
                    }
                    media.position += 1;
                }
            } else {
                let mut fm_seen = 0i32;
                while fm_seen < -count {
                    if media.position == 0 {
                        return ScsiResult {
                            status: 0x02,
                            data_in: Vec::new(),
                            sense: build_sense(0x00, 0x00, 0x04), // BOM
                        };
                    }
                    media.position -= 1;
                    if matches!(media.records[media.position], TapeRecord::Filemark) {
                        fm_seen += 1;
                    }
                }
            }
        }
        3 => {
            // Space to end-of-data
            media.position = media.records.len();
        }
        _ => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense(0x05, 0x24, 0x00), // INVALID FIELD IN CDB
            };
        }
    }

    trace!(code, count, position = media.position, "SPACE complete");

    ScsiResult {
        status: 0x00,
        data_in: Vec::new(),
        sense: Vec::new(),
    }
}

/// READ POSITION — short form, 20 bytes
fn handle_read_position(state: &Mutex<DriveState>) -> ScsiResult {
    let st = state.lock().unwrap();
    let media = match st.media.as_ref() {
        Some(m) => m,
        None => {
            return ScsiResult {
                status: 0x02,
                data_in: Vec::new(),
                sense: build_sense_no_media(),
            }
        }
    };

    let pos = media.position as u32;
    let mut data = vec![0u8; 20];

    // Byte 0: flags — BOP (bit 7), EOP (bit 6)
    if media.position == 0 {
        data[0] |= 0x80; // BOP
    }
    if media.position >= media.records.len() && !media.records.is_empty() {
        data[0] |= 0x40; // EOP
    }

    // Bytes 4-7: first block location (BE32)
    data[4] = ((pos >> 24) & 0xFF) as u8;
    data[5] = ((pos >> 16) & 0xFF) as u8;
    data[6] = ((pos >> 8) & 0xFF) as u8;
    data[7] = (pos & 0xFF) as u8;

    // Bytes 8-11: last block location (same for our purposes)
    data[8] = data[4];
    data[9] = data[5];
    data[10] = data[6];
    data[11] = data[7];

    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// READ BLOCK LIMITS — 6 bytes
fn handle_read_block_limits() -> ScsiResult {
    let mut data = vec![0u8; 6];
    // Byte 0: granularity = 0
    // Bytes 1-3: maximum block length (BE24) = 256KB
    data[1] = ((MAX_BLOCK_SIZE >> 16) & 0xFF) as u8;
    data[2] = ((MAX_BLOCK_SIZE >> 8) & 0xFF) as u8;
    data[3] = (MAX_BLOCK_SIZE & 0xFF) as u8;
    // Bytes 4-5: minimum block length (BE16) = 1
    data[4] = 0x00;
    data[5] = 0x01;

    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// MODE SENSE(6) — 4-byte header + 8-byte block descriptor (variable block mode)
fn handle_mode_sense_6(cdb: &[u8], state: &Mutex<DriveState>) -> ScsiResult {
    let alloc_len = cdb[4] as usize;
    let st = state.lock().unwrap();

    // 4-byte mode parameter header + 8-byte block descriptor = 12 bytes
    let mut data = vec![0u8; 12];

    // Header
    data[0] = 11; // Mode data length (everything after byte 0)
    data[1] = 0x00; // Medium type: default
    // Byte 2: device-specific parameter
    //   bit 7 = WP (write-protected) = 0
    //   bit 4 = BUFFERED MODE = 1 (buffered)
    data[2] = 0x10;
    data[3] = 0x08; // Block descriptor length = 8

    // Block descriptor (8 bytes)
    // Byte 0: density code (0x00 = default)
    if st.media.is_some() {
        data[4] = 0x00; // density code
    }
    // Bytes 1-3: number of blocks = 0 (variable)
    // Byte 4: reserved
    // Bytes 5-7: block length = 0 (variable block mode)

    data.truncate(alloc_len);

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

fn build_sense_no_media() -> Vec<u8> {
    build_sense(0x02, 0x3A, 0x00) // NOT READY: MEDIUM NOT PRESENT
}
