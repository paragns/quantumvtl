use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tracing::{debug, error, info, trace, warn};

use crate::login::LoginNegotiator;
use crate::pdu::{self, Pdu};
use crate::session::{SessionGuard, SessionInfo, SessionRegistry};
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
    pub registry: Arc<SessionRegistry>,
}

impl TargetServer {
    pub fn new(target: Target, registry: Arc<SessionRegistry>) -> Self {
        Self {
            target: Arc::new(target),
            registry,
        }
    }

    /// Run the iSCSI target server, listening on the given address.
    /// Stops when `shutdown` is notified.
    pub async fn run(self, addr: &str, shutdown: Arc<Notify>) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!(addr, target = %self.target.name, "iSCSI target listening");
        self.run_on(listener, shutdown).await
    }

    /// Run the iSCSI target server on a pre-bound listener.
    /// Stops when `shutdown` is notified.
    pub async fn run_on(
        self,
        listener: TcpListener,
        shutdown: Arc<Notify>,
    ) -> std::io::Result<()> {
        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, peer) = accept?;
                    info!(%peer, "new iSCSI connection");
                    let target = self.target.clone();
                    let shutdown = shutdown.clone();
                    let registry = self.registry.clone();
                    let peer_str = peer.to_string();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, target, shutdown, registry, &peer_str).await {
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

/// Maximum number of SCSI commands in flight per connection.
const MAX_INFLIGHT: u64 = 32;

/// State for a completed SCSI command awaiting response send.
struct CommandCompletion {
    seq: u64,
    itt: u32,
    read_bit: bool,
    edtl: u32,
    result: ScsiResult,
}

/// Handle a single iSCSI connection.
async fn handle_connection(
    stream: TcpStream,
    target: Arc<Target>,
    shutdown: Arc<Notify>,
    registry: Arc<SessionRegistry>,
    peer_addr: &str,
) -> std::io::Result<()> {
    stream.set_nodelay(true)?;
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let mut login = LoginNegotiator::new(target.name.clone());
    let mut stat_sn: u32 = 0;
    let mut exp_cmd_sn: u32 = 1;
    let mut guard = SessionGuard::new(registry.clone());

    // ── Login phase ─────────────────────────────────────────────────────
    loop {
        let req = tokio::select! {
            result = Pdu::read_from(&mut reader) => {
                match result {
                    Ok(pdu) => pdu,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        debug!("connection closed by initiator during login");
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            _ = shutdown.notified() => {
                debug!("connection shutdown by server during login");
                return Ok(());
            }
        };

        let opcode = req.opcode();
        match opcode {
            pdu::OPCODE_LOGIN_REQ => {
                let resp = login.handle_login(&req, stat_sn, exp_cmd_sn);
                stat_sn = stat_sn.wrapping_add(1);

                let login_complete =
                    resp.bhs[36] == 0 && resp.login_transit() && resp.login_nsg() == 3;
                resp.write_to(&mut writer).await?;

                if login_complete {
                    let tsih = login.tsih;
                    let initiator = login
                        .initiator_name
                        .clone()
                        .unwrap_or_else(|| "unknown".into());
                    info!(initiator = %initiator, tsih, "login complete, entering Full Feature Phase");
                    registry.register(
                        tsih,
                        SessionInfo {
                            initiator_name: initiator,
                            tsih,
                            peer_addr: peer_addr.to_string(),
                            connected_since: chrono::Utc::now()
                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                            active_commands: 0,
                        },
                    );
                    guard.set_tsih(tsih);
                    break;
                }
            }
            _ => {
                warn!(opcode, "unexpected opcode during login phase, ignoring");
            }
        }
    }

    // ── Full Feature Phase ──────────────────────────────────────────────
    run_full_feature_phase(
        &mut reader,
        &mut writer,
        &target,
        &shutdown,
        &login,
        &mut stat_sn,
        &mut exp_cmd_sn,
    )
    .await
}

/// Collect write data for a SCSI write command.
///
/// Handles unsolicited Data-Out PDUs (when `InitialR2T=No`) and R2T-solicited
/// Data-Out sequences.
#[allow(clippy::too_many_arguments)]
async fn collect_write_data(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    req: &Pdu,
    initial_r2t: bool,
    first_burst_length: usize,
    edtl: u32,
    lun_raw: u64,
    itt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
) -> std::io::Result<Vec<u8>> {
    let mut data = req.data.clone();

    // Unsolicited Data-Out (InitialR2T=No)
    if !initial_r2t && (data.len() as u32) < edtl {
        let unsolicited_limit = first_burst_length;
        loop {
            let data_pdu = Pdu::read_from(reader).await?;
            if data_pdu.opcode() != pdu::OPCODE_DATA_OUT {
                warn!(opcode = data_pdu.opcode(), "expected unsolicited Data-Out PDU");
                break;
            }

            let buf_offset = data_pdu.buffer_offset() as usize;
            let chunk = &data_pdu.data;
            if buf_offset + chunk.len() > data.len() {
                data.resize(buf_offset + chunk.len(), 0);
            }
            data[buf_offset..buf_offset + chunk.len()].copy_from_slice(chunk);

            if data_pdu.flags() & 0x80 != 0 {
                break;
            }
            if data.len() >= unsolicited_limit {
                break;
            }
        }
    }

    // R2T-solicited Data-Out
    if (data.len() as u32) < edtl {
        let remaining = edtl - data.len() as u32;
        let ttt = itt;
        let r2t = pdu::build_r2t(
            lun_raw,
            itt,
            ttt,
            stat_sn,
            exp_cmd_sn,
            exp_cmd_sn.wrapping_add(32),
            0,
            data.len() as u32,
            remaining,
        );
        r2t.write_to(writer).await?;

        loop {
            let data_pdu = Pdu::read_from(reader).await?;
            if data_pdu.opcode() != pdu::OPCODE_DATA_OUT {
                warn!(opcode = data_pdu.opcode(), "expected Data-Out PDU during R2T");
                break;
            }

            let buf_offset = data_pdu.buffer_offset() as usize;
            let chunk = &data_pdu.data;
            if buf_offset + chunk.len() > data.len() {
                data.resize(buf_offset + chunk.len(), 0);
            }
            data[buf_offset..buf_offset + chunk.len()].copy_from_slice(chunk);

            if data_pdu.flags() & 0x80 != 0 {
                break;
            }
        }
    }

    Ok(data)
}

/// Send response PDUs for a completed SCSI command.
async fn send_command_response(
    writer: &mut BufWriter<OwnedWriteHalf>,
    completion: CommandCompletion,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_recv_data_segment_length: usize,
) -> std::io::Result<()> {
    if completion.read_bit && !completion.result.data_in.is_empty() {
        let mut data = completion.result.data_in;
        if data.len() > completion.edtl as usize {
            data.truncate(completion.edtl as usize);
        }
        let pdus = pdu::build_data_in_sequence(
            completion.itt,
            stat_sn,
            exp_cmd_sn,
            exp_cmd_sn.wrapping_add(32),
            completion.result.status,
            data,
            max_recv_data_segment_length,
        );
        for p in pdus {
            p.write_to(writer).await?;
        }
    } else {
        let resp = pdu::build_scsi_response(
            completion.itt,
            stat_sn,
            exp_cmd_sn,
            exp_cmd_sn.wrapping_add(32),
            completion.result.status,
            completion.result.sense,
        );
        resp.write_to(writer).await?;
    }
    Ok(())
}

/// Run the Full Feature Phase with multiplexed command processing.
///
/// Commands to different LUNs execute in parallel via `spawn_blocking`.
/// Responses are sent in StatSN order using a reorder buffer.
async fn run_full_feature_phase(
    reader: &mut BufReader<OwnedReadHalf>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    target: &Arc<Target>,
    shutdown: &Arc<Notify>,
    login: &LoginNegotiator,
    stat_sn: &mut u32,
    exp_cmd_sn: &mut u32,
) -> std::io::Result<()> {
    let (completion_tx, mut completion_rx) =
        tokio::sync::mpsc::unbounded_channel::<CommandCompletion>();
    let mut pending: BTreeMap<u64, CommandCompletion> = BTreeMap::new();
    let mut next_dispatch_seq: u64 = 0;
    let mut next_send_seq: u64 = 0;
    let max_recv = login.initiator_max_recv_data_segment_length;
    let mut reading = true;
    let mut logout_itt: Option<u32> = None;

    loop {
        // All in-flight commands drained — safe to exit
        if !reading && next_send_seq == next_dispatch_seq {
            if let Some(itt) = logout_itt {
                let resp = pdu::build_logout_response(
                    itt,
                    *stat_sn,
                    *exp_cmd_sn,
                    exp_cmd_sn.wrapping_add(32),
                );
                resp.write_to(writer).await?;
                info!("logout complete");
            }
            return Ok(());
        }

        let can_read = reading && (next_dispatch_seq - next_send_seq < MAX_INFLIGHT);

        tokio::select! {
            biased;

            // Priority: drain completions to free in-flight slots
            Some(completion) = completion_rx.recv() => {
                pending.insert(completion.seq, completion);
                while let Some(comp) = pending.remove(&next_send_seq) {
                    send_command_response(writer, comp, *stat_sn, *exp_cmd_sn, max_recv).await?;
                    *stat_sn = stat_sn.wrapping_add(1);
                    next_send_seq += 1;
                }
            }

            // Read next PDU from initiator (gated by in-flight limit)
            result = Pdu::read_from(reader), if can_read => {
                let req = match result {
                    Ok(pdu) => pdu,
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        debug!("connection closed by initiator");
                        reading = false;
                        continue;
                    }
                    Err(e) => return Err(e),
                };

                let opcode = req.opcode();
                trace!(opcode, itt = req.itt(), "received PDU");

                match opcode {
                    pdu::OPCODE_SCSI_CMD => {
                        let lun_raw = req.lun();
                        let lun = decode_lun(lun_raw);
                        let cdb_bytes: [u8; 16] = req.cdb().try_into().unwrap();
                        let itt = req.itt();
                        let edtl = req.expected_data_transfer_length();
                        let flags = req.flags();
                        let read_bit = flags & 0x40 != 0;
                        let write_bit = flags & 0x20 != 0;

                        if !req.is_immediate() {
                            *exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                        }

                        debug!(
                            lun,
                            cdb0 = cdb_bytes[0],
                            edtl,
                            read = read_bit,
                            write = write_bit,
                            "SCSI command"
                        );

                        // Collect write data (R2T exchange happens inline on this connection)
                        let write_data = if write_bit && edtl > 0 {
                            collect_write_data(
                                reader,
                                writer,
                                &req,
                                login.initial_r2t,
                                login.first_burst_length,
                                edtl,
                                lun_raw,
                                itt,
                                *stat_sn,
                                *exp_cmd_sn,
                            )
                            .await?
                        } else {
                            req.data.clone()
                        };

                        let seq = next_dispatch_seq;
                        next_dispatch_seq += 1;

                        let device = if cdb_bytes[0] == 0xA0 {
                            None // REPORT LUNS — handled at target level
                        } else {
                            target.luns.get(&lun).cloned()
                        };
                        let target_ref = target.clone();
                        let tx = completion_tx.clone();

                        tokio::task::spawn_blocking(move || {
                            let result = std::panic::catch_unwind(
                                std::panic::AssertUnwindSafe(|| {
                                    if cdb_bytes[0] == 0xA0 {
                                        handle_report_luns(&target_ref, &cdb_bytes)
                                    } else if let Some(dev) = device {
                                        dev.execute_command(&cdb_bytes, &write_data)
                                    } else {
                                        warn!(lun, "unknown LUN");
                                        ScsiResult {
                                            status: 0x02,
                                            data_in: Vec::new(),
                                            sense: build_sense(0x05, 0x25, 0x00),
                                        }
                                    }
                                }),
                            );

                            let result = match result {
                                Ok(r) => r,
                                Err(panic_info) => {
                                    let msg =
                                        if let Some(s) = panic_info.downcast_ref::<&str>() {
                                            s.to_string()
                                        } else if let Some(s) =
                                            panic_info.downcast_ref::<String>()
                                        {
                                            s.clone()
                                        } else {
                                            "unknown panic".to_string()
                                        };
                                    error!(lun, itt, seq, %msg, "SCSI command panicked");
                                    ScsiResult {
                                        status: 0x02, // CHECK CONDITION
                                        data_in: Vec::new(),
                                        sense: build_sense(0x04, 0x44, 0x00), // INTERNAL TARGET FAILURE
                                    }
                                }
                            };

                            let _ = tx.send(CommandCompletion {
                                seq,
                                itt,
                                read_bit,
                                edtl,
                                result,
                            });
                        });
                    }

                    pdu::OPCODE_NOP_OUT => {
                        let itt = req.itt();
                        let ttt = req.ttt();
                        if !req.is_immediate() {
                            *exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                        }
                        let resp = pdu::build_nop_in(
                            itt,
                            ttt,
                            *stat_sn,
                            *exp_cmd_sn,
                            exp_cmd_sn.wrapping_add(32),
                        );
                        *stat_sn = stat_sn.wrapping_add(1);
                        resp.write_to(writer).await?;
                    }

                    pdu::OPCODE_TEXT_REQ => {
                        let itt = req.itt();
                        if !req.is_immediate() {
                            *exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                        }
                        let kv = crate::login::parse_kv_pairs(&req.data);
                        let mut resp_data = Vec::new();
                        for (k, v) in &kv {
                            if k == "SendTargets" && (v == "All" || v == &target.name) {
                                let entry = format!("TargetName={}\0", target.name);
                                resp_data.extend_from_slice(entry.as_bytes());
                            }
                        }
                        let resp = pdu::build_text_response(
                            itt,
                            0xFFFFFFFF,
                            *stat_sn,
                            *exp_cmd_sn,
                            exp_cmd_sn.wrapping_add(32),
                            resp_data,
                        );
                        *stat_sn = stat_sn.wrapping_add(1);
                        resp.write_to(writer).await?;
                    }

                    pdu::OPCODE_LOGOUT_REQ => {
                        let itt = req.itt();
                        if !req.is_immediate() {
                            *exp_cmd_sn = exp_cmd_sn.wrapping_add(1);
                        }
                        logout_itt = Some(itt);
                        reading = false;
                    }

                    _ => {
                        warn!(opcode, "unhandled PDU opcode in FFP, ignoring");
                    }
                }
            }

            _ = shutdown.notified() => {
                debug!("connection shutdown by server");
                return Ok(());
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
