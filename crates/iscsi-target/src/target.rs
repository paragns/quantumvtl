use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tracing::{debug, error, info, trace, warn};

use crate::login::LoginNegotiator;
use crate::pdu::{self, Pdu};
use crate::{ScsiDevice, ScsiResult};

/// An iSCSI target with a name and a set of LUNs.
pub struct Target {
    pub name: String,
    pub luns: HashMap<u64, Arc<dyn ScsiDevice>>,
}

impl Target {
    pub fn new(name: String) -> Self {
        Self {
            name,
            luns: HashMap::new(),
        }
    }

    pub fn add_lun(&mut self, lun: u64, device: Arc<dyn ScsiDevice>) {
        self.luns.insert(lun, device);
    }
}

/// iSCSI target server that listens for connections.
pub struct TargetServer {
    target: Arc<Target>,
}

impl TargetServer {
    pub fn new(target: Target) -> Self {
        Self {
            target: Arc::new(target),
        }
    }

    /// Run the iSCSI target server, listening on the given address.
    /// Stops when `shutdown` is notified.
    pub async fn run(
        self,
        addr: &str,
        shutdown: Arc<Notify>,
    ) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(addr, target = %self.target.name, "iSCSI target listening");

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, peer) = accept?;
                    info!(%peer, "new iSCSI connection");
                    let target = self.target.clone();
                    let shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, target, shutdown).await {
                            error!(%peer, "connection error: {e}");
                        }
                        debug!(%peer, "connection closed");
                    });
                }
                _ = shutdown.notified() => {
                    info!("iSCSI target shutting down");
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Decode a LUN from the 8-byte LUN field per SAM-2.
/// Supports single-level LUN addressing (format 00b).
fn decode_lun(raw: u64) -> u64 {
    let bytes = raw.to_be_bytes();
    // First two bytes contain the LUN for single-level addressing
    let address_method = (bytes[0] >> 6) & 0x03;
    if address_method == 0 {
        // Peripheral device addressing: LUN in bits 5-0 of byte 1
        // Actually for simple LUN: byte 0 bits 5-0 (bus) and byte 1 (LUN)
        ((bytes[0] as u64 & 0x3F) << 8) | (bytes[1] as u64)
    } else {
        // Flat space addressing or others — just use byte 1
        bytes[1] as u64
    }
}

/// Handle a single iSCSI connection.
async fn handle_connection(
    stream: TcpStream,
    target: Arc<Target>,
    shutdown: Arc<Notify>,
) -> std::io::Result<()> {
    stream.set_nodelay(true)?;
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let mut login = LoginNegotiator::new(target.name.clone());
    let mut stat_sn: u32 = 0;
    let mut exp_cmd_sn: u32 = 1;
    let mut logged_in = false;

    loop {
        let req = tokio::select! {
            result = Pdu::read_from(&mut reader) => {
                match result {
                    Ok(pdu) => pdu,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        debug!("connection closed by initiator");
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            _ = shutdown.notified() => {
                debug!("connection shutdown by server");
                return Ok(());
            }
        };

        let opcode = req.opcode();
        trace!(opcode, itt = req.itt(), "received PDU");

        match opcode {
            pdu::OPCODE_LOGIN_REQ => {
                let resp = login.handle_login(&req, stat_sn, exp_cmd_sn);
                stat_sn = stat_sn.wrapping_add(1);

                // Check if login completed (transit to FFP, status=success)
                if resp.bhs[36] == 0 && resp.login_transit() && resp.login_nsg() == 3 {
                    logged_in = true;
                    info!("login complete, entering Full Feature Phase");
                }

                resp.write_to(&mut writer).await?;
            }

            pdu::OPCODE_SCSI_CMD if logged_in => {
                let lun_raw = req.lun();
                let lun = decode_lun(lun_raw);
                let cdb = req.cdb();
                let itt = req.itt();
                let edtl = req.expected_data_transfer_length();
                let flags = req.flags();
                let read_bit = flags & 0x40 != 0;

                // Track CmdSN
                if !req.is_immediate() {
                    exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                }

                debug!(lun, cdb0 = cdb[0], edtl, read = read_bit, "SCSI command");

                let result = if cdb[0] == 0xA0 {
                    // REPORT LUNS — handle at the target level
                    handle_report_luns(&target, cdb)
                } else if let Some(device) = target.luns.get(&lun) {
                    device.execute_command(cdb, &req.data)
                } else {
                    warn!(lun, "unknown LUN");
                    // ILLEGAL REQUEST: LOGICAL UNIT NOT SUPPORTED
                    ScsiResult {
                        status: 0x02,
                        data_in: Vec::new(),
                        sense: build_sense(0x05, 0x25, 0x00),
                    }
                };

                let resp = if read_bit && !result.data_in.is_empty() {
                    // Truncate data to EDTL
                    let mut data = result.data_in;
                    if data.len() > edtl as usize {
                        data.truncate(edtl as usize);
                    }
                    pdu::build_data_in(
                        itt, stat_sn, exp_cmd_sn, exp_cmd_sn.wrapping_add(32),
                        result.status, data,
                    )
                } else {
                    pdu::build_scsi_response(
                        itt, stat_sn, exp_cmd_sn, exp_cmd_sn.wrapping_add(32),
                        result.status, result.sense,
                    )
                };
                stat_sn = stat_sn.wrapping_add(1);
                resp.write_to(&mut writer).await?;
            }

            pdu::OPCODE_NOP_OUT if logged_in => {
                let itt = req.itt();
                let ttt = req.ttt();
                if !req.is_immediate() {
                    exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                }
                let resp = pdu::build_nop_in(
                    itt, ttt, stat_sn, exp_cmd_sn, exp_cmd_sn.wrapping_add(32),
                );
                stat_sn = stat_sn.wrapping_add(1);
                resp.write_to(&mut writer).await?;
            }

            pdu::OPCODE_TEXT_REQ if logged_in => {
                let itt = req.itt();
                if !req.is_immediate() {
                    exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                }
                // Handle SendTargets discovery
                let kv = crate::login::parse_kv_pairs(&req.data);
                let mut resp_data = Vec::new();
                for (k, v) in &kv {
                    if k == "SendTargets" && (v == "All" || v == &target.name) {
                        let entry = format!("TargetName={}\0", target.name);
                        resp_data.extend_from_slice(entry.as_bytes());
                    }
                }
                let resp = pdu::build_text_response(
                    itt, 0xFFFFFFFF, stat_sn, exp_cmd_sn, exp_cmd_sn.wrapping_add(32),
                    resp_data,
                );
                stat_sn = stat_sn.wrapping_add(1);
                resp.write_to(&mut writer).await?;
            }

            pdu::OPCODE_LOGOUT_REQ => {
                let itt = req.itt();
                if !req.is_immediate() {
                    exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                }
                let resp = pdu::build_logout_response(
                    itt, stat_sn, exp_cmd_sn, exp_cmd_sn.wrapping_add(32),
                );
                resp.write_to(&mut writer).await?;
                info!("logout complete");
                return Ok(());
            }

            _ => {
                warn!(opcode, logged_in, "unhandled PDU opcode, ignoring");
            }
        }
    }
}

/// Handle REPORT LUNS (opcode 0xA0) at the target level.
fn handle_report_luns(target: &Target, cdb: &[u8]) -> ScsiResult {
    let alloc_len = u32::from_be_bytes([cdb[6], cdb[7], cdb[8], cdb[9]]) as usize;

    let lun_count = target.luns.len();
    let lun_list_length = lun_count * 8;

    // Response: 4-byte LUN list length + 4 reserved + 8 bytes per LUN
    let mut data = Vec::with_capacity(8 + lun_list_length);
    data.extend_from_slice(&(lun_list_length as u32).to_be_bytes());
    data.extend_from_slice(&[0u8; 4]); // reserved

    // Add each LUN in SAM LUN format
    let mut luns: Vec<u64> = target.luns.keys().copied().collect();
    luns.sort();
    for lun in luns {
        // Single-level LUN addressing: bus=0, LUN in second byte
        let mut lun_bytes = [0u8; 8];
        lun_bytes[1] = lun as u8;
        data.extend_from_slice(&lun_bytes);
    }

    if data.len() > alloc_len {
        data.truncate(alloc_len);
    }

    ScsiResult {
        status: 0x00,
        data_in: data,
        sense: Vec::new(),
    }
}

/// Build a fixed-format sense data blob.
fn build_sense(sense_key: u8, asc: u8, ascq: u8) -> Vec<u8> {
    let mut sense = vec![0u8; 18];
    sense[0] = 0x70; // Response code: current, fixed format
    sense[2] = sense_key & 0x0F;
    sense[7] = 10; // Additional sense length
    sense[12] = asc;
    sense[13] = ascq;
    sense
}
