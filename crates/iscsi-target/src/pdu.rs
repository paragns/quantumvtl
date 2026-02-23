use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

/// iSCSI opcodes (initiator → target).
pub const OPCODE_NOP_OUT: u8 = 0x00;
pub const OPCODE_SCSI_CMD: u8 = 0x01;
pub const OPCODE_LOGIN_REQ: u8 = 0x03;
pub const OPCODE_TEXT_REQ: u8 = 0x04;
pub const OPCODE_DATA_OUT: u8 = 0x05;
pub const OPCODE_LOGOUT_REQ: u8 = 0x06;

/// iSCSI opcodes (target → initiator).
pub const OPCODE_NOP_IN: u8 = 0x20;
pub const OPCODE_SCSI_RESP: u8 = 0x21;
pub const OPCODE_R2T: u8 = 0x31;
pub const OPCODE_SCSI_DATA_IN: u8 = 0x25;
pub const OPCODE_LOGIN_RESP: u8 = 0x23;
pub const OPCODE_TEXT_RESP: u8 = 0x24;
pub const OPCODE_LOGOUT_RESP: u8 = 0x26;

/// BHS size in bytes.
pub const BHS_SIZE: usize = 48;

/// A raw iSCSI PDU: 48-byte Basic Header Segment + optional data segment.
#[derive(Debug, Clone)]
pub struct Pdu {
    pub bhs: [u8; BHS_SIZE],
    pub data: Vec<u8>,
}

impl Default for Pdu {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdu {
    pub fn new() -> Self {
        Self {
            bhs: [0u8; BHS_SIZE],
            data: Vec::new(),
        }
    }

    // --- BHS accessor helpers ---

    pub fn opcode(&self) -> u8 {
        self.bhs[0] & 0x3F
    }

    pub fn set_opcode(&mut self, op: u8) {
        self.bhs[0] = (self.bhs[0] & 0xC0) | (op & 0x3F);
    }

    pub fn is_immediate(&self) -> bool {
        self.bhs[0] & 0x40 != 0
    }

    /// Byte 1 flags (F, C, R, W, etc. depending on opcode).
    pub fn flags(&self) -> u8 {
        self.bhs[1]
    }

    pub fn set_flags(&mut self, f: u8) {
        self.bhs[1] = f;
    }

    /// Total AHS length (bytes 4) in 4-byte words.
    pub fn total_ahs_length(&self) -> usize {
        self.bhs[4] as usize
    }

    /// Data segment length (bytes 5-7), big-endian 24-bit.
    pub fn data_segment_length(&self) -> usize {
        ((self.bhs[5] as usize) << 16) | ((self.bhs[6] as usize) << 8) | (self.bhs[7] as usize)
    }

    pub fn set_data_segment_length(&mut self, len: usize) {
        self.bhs[5] = ((len >> 16) & 0xFF) as u8;
        self.bhs[6] = ((len >> 8) & 0xFF) as u8;
        self.bhs[7] = (len & 0xFF) as u8;
    }

    /// LUN field (bytes 8-15). For SCSI Command PDUs.
    pub fn lun(&self) -> u64 {
        u64::from_be_bytes(self.bhs[8..16].try_into().unwrap())
    }

    pub fn set_lun(&mut self, lun: u64) {
        self.bhs[8..16].copy_from_slice(&lun.to_be_bytes());
    }

    /// Initiator Task Tag (bytes 16-19).
    pub fn itt(&self) -> u32 {
        u32::from_be_bytes(self.bhs[16..20].try_into().unwrap())
    }

    pub fn set_itt(&mut self, itt: u32) {
        self.bhs[16..20].copy_from_slice(&itt.to_be_bytes());
    }

    /// Target Transfer Tag (bytes 20-23).
    pub fn ttt(&self) -> u32 {
        u32::from_be_bytes(self.bhs[20..24].try_into().unwrap())
    }

    pub fn set_ttt(&mut self, ttt: u32) {
        self.bhs[20..24].copy_from_slice(&ttt.to_be_bytes());
    }

    /// CmdSN (bytes 24-27) — used in login/command PDUs.
    pub fn cmd_sn(&self) -> u32 {
        u32::from_be_bytes(self.bhs[24..28].try_into().unwrap())
    }

    pub fn set_cmd_sn(&mut self, sn: u32) {
        self.bhs[24..28].copy_from_slice(&sn.to_be_bytes());
    }

    /// ExpStatSN (bytes 28-31) — used in initiator PDUs.
    pub fn exp_stat_sn(&self) -> u32 {
        u32::from_be_bytes(self.bhs[28..32].try_into().unwrap())
    }

    /// StatSN (bytes 24-27) — target→initiator response PDUs.
    pub fn set_stat_sn(&mut self, sn: u32) {
        self.bhs[24..28].copy_from_slice(&sn.to_be_bytes());
    }

    /// ExpCmdSN (bytes 28-31) — target→initiator response PDUs.
    pub fn set_exp_cmd_sn(&mut self, sn: u32) {
        self.bhs[28..32].copy_from_slice(&sn.to_be_bytes());
    }

    /// MaxCmdSN (bytes 32-35) — target→initiator response PDUs.
    pub fn set_max_cmd_sn(&mut self, sn: u32) {
        self.bhs[32..36].copy_from_slice(&sn.to_be_bytes());
    }

    /// For SCSI Command PDUs: Expected Data Transfer Length (bytes 20-23 is TTT,
    /// but for SCSI CMD it's bytes 20-23 = TTT area isn't used; EDTL is bytes 20-23?
    /// Actually per RFC 3720: SCSI Command PDU byte 20-23 = EDTL. TTT is not in SCSI CMD.
    /// Wait — let me re-check: BHS layout for SCSI Command (opcode 0x01):
    ///   0: opcode, 1: flags (F,R,W,ATTR), 2-3: reserved
    ///   4: total AHS length, 5-7: data segment length
    ///   8-15: LUN, 16-19: ITT
    ///   20-23: Expected Data Transfer Length
    ///   24-27: CmdSN, 28-31: ExpStatSN
    ///   32-47: CDB (16 bytes)
    pub fn expected_data_transfer_length(&self) -> u32 {
        u32::from_be_bytes(self.bhs[20..24].try_into().unwrap())
    }

    /// CDB bytes for SCSI Command PDU (bytes 32-47).
    pub fn cdb(&self) -> &[u8] {
        &self.bhs[32..48]
    }

    /// ISID (bytes 8-13) — used in Login PDUs.
    pub fn isid(&self) -> [u8; 6] {
        self.bhs[8..14].try_into().unwrap()
    }

    pub fn set_isid(&mut self, isid: &[u8; 6]) {
        self.bhs[8..14].copy_from_slice(isid);
    }

    /// TSIH (bytes 14-15) — used in Login PDUs.
    pub fn tsih(&self) -> u16 {
        u16::from_be_bytes(self.bhs[14..16].try_into().unwrap())
    }

    pub fn set_tsih(&mut self, tsih: u16) {
        self.bhs[14..16].copy_from_slice(&tsih.to_be_bytes());
    }

    /// CSG (Current Stage) from Login flags byte 1, bits 2-3.
    pub fn login_csg(&self) -> u8 {
        (self.bhs[1] >> 2) & 0x03
    }

    /// NSG (Next Stage) from Login flags byte 1, bits 0-1.
    pub fn login_nsg(&self) -> u8 {
        self.bhs[1] & 0x03
    }

    /// Transit bit (T) from Login flags byte 1, bit 7.
    pub fn login_transit(&self) -> bool {
        self.bhs[1] & 0x80 != 0
    }

    /// Buffer Offset (bytes 40-43) — in R2T and Data-Out PDUs.
    pub fn buffer_offset(&self) -> u32 {
        u32::from_be_bytes(self.bhs[40..44].try_into().unwrap())
    }

    pub fn set_buffer_offset(&mut self, offset: u32) {
        self.bhs[40..44].copy_from_slice(&offset.to_be_bytes());
    }

    /// Desired Data Transfer Length (bytes 44-47) — in R2T PDUs.
    pub fn set_desired_data_transfer_length(&mut self, len: u32) {
        self.bhs[44..48].copy_from_slice(&len.to_be_bytes());
    }

    /// R2TSN (bytes 36-39) — in R2T PDUs.
    pub fn set_r2t_sn(&mut self, sn: u32) {
        self.bhs[36..40].copy_from_slice(&sn.to_be_bytes());
    }

    /// DataSN (bytes 36-39) — in Data-Out PDUs.
    pub fn data_sn(&self) -> u32 {
        u32::from_be_bytes(self.bhs[36..40].try_into().unwrap())
    }

    /// Read a PDU from an async reader.
    pub async fn read_from<R: AsyncReadExt + Unpin>(reader: &mut R) -> std::io::Result<Self> {
        let mut bhs = [0u8; BHS_SIZE];
        reader.read_exact(&mut bhs).await?;

        let ahs_len = (bhs[4] as usize) * 4;
        let data_len =
            ((bhs[5] as usize) << 16) | ((bhs[6] as usize) << 8) | (bhs[7] as usize);
        let total_payload = ahs_len + data_len;
        // Pad to 4-byte boundary
        let padded = (total_payload + 3) & !3;

        let mut data = vec![0u8; padded];
        if padded > 0 {
            reader.read_exact(&mut data).await?;
        }
        // Truncate to actual data length (skip AHS, drop padding)
        let data = if data_len > 0 {
            data[ahs_len..ahs_len + data_len].to_vec()
        } else {
            Vec::new()
        };

        let opcode = bhs[0] & 0x3F;
        trace!(opcode = opcode, data_len = data_len, "read PDU");

        Ok(Self { bhs, data })
    }

    /// Write a PDU to an async writer.
    pub async fn write_to<W: AsyncWriteExt + Unpin>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.bhs).await?;
        if !self.data.is_empty() {
            writer.write_all(&self.data).await?;
            // Pad to 4-byte boundary
            let pad = (4 - (self.data.len() % 4)) % 4;
            if pad > 0 {
                writer.write_all(&vec![0u8; pad]).await?;
            }
        }
        writer.flush().await?;
        Ok(())
    }
}

// --- Builder functions for response PDUs ---

/// Build a Login Response PDU.
#[allow(clippy::too_many_arguments)]
pub fn build_login_response(
    req: &Pdu,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
    csg: u8,
    nsg: u8,
    transit: bool,
    tsih: u16,
    status_class: u8,
    status_detail: u8,
    data: Vec<u8>,
) -> Pdu {
    let mut pdu = Pdu::new();
    let mut flags = ((csg & 0x03) << 2) | (nsg & 0x03);
    if transit {
        flags |= 0x80; // T bit
    }
    pdu.set_opcode(OPCODE_LOGIN_RESP);
    pdu.set_flags(flags);
    // Version-max and Version-min (byte 2-3)
    pdu.bhs[2] = 0x00; // Version-max
    pdu.bhs[3] = 0x00; // Version-min
    pdu.set_data_segment_length(data.len());
    pdu.set_isid(&req.isid());
    pdu.set_tsih(tsih);
    pdu.set_itt(req.itt());
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    // Status-Class (byte 36), Status-Detail (byte 37)
    pdu.bhs[36] = status_class;
    pdu.bhs[37] = status_detail;
    pdu.data = data;
    pdu
}

/// Build a SCSI Data-In PDU (opcode 0x25) with status piggybacked.
pub fn build_data_in(
    itt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
    scsi_status: u8,
    data: Vec<u8>,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_SCSI_DATA_IN);
    // F=1 (Final), S=1 (Status piggybacked) → flags = 0x80 | 0x01 = 0x81
    pdu.set_flags(0x81);
    pdu.set_data_segment_length(data.len());
    pdu.set_itt(itt);
    pdu.set_ttt(0xFFFFFFFF); // no TTT
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    // SCSI status in byte 3 for Data-In
    pdu.bhs[3] = scsi_status;
    // Residual count = 0 (bytes 44-47)
    pdu.data = data;
    pdu
}

/// Build a SCSI Response PDU (opcode 0x21) for commands with no data-in.
pub fn build_scsi_response(
    itt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
    scsi_status: u8,
    sense_data: Vec<u8>,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_SCSI_RESP);
    // Flags: F=1 (0x80)
    pdu.set_flags(0x80);
    // Response code: 0x00 = command completed at target
    pdu.bhs[2] = 0x00;
    // SCSI status
    pdu.bhs[3] = scsi_status;
    pdu.set_itt(itt);
    pdu.set_ttt(0xFFFFFFFF);
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);

    if !sense_data.is_empty() {
        // Sense data is prefixed with 2-byte length in the data segment
        let mut data = Vec::with_capacity(2 + sense_data.len());
        let sense_len = sense_data.len() as u16;
        data.extend_from_slice(&sense_len.to_be_bytes());
        data.extend_from_slice(&sense_data);
        pdu.set_data_segment_length(data.len());
        pdu.data = data;
    }

    pdu
}

/// Build a NOP-In PDU (response to NOP-Out).
pub fn build_nop_in(
    itt: u32,
    ttt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_NOP_IN);
    pdu.set_flags(0x80); // F=1
    pdu.set_itt(itt);
    pdu.set_ttt(ttt);
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    pdu
}

/// Build a Logout Response PDU.
pub fn build_logout_response(
    itt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_LOGOUT_RESP);
    pdu.set_flags(0x80); // F=1
    // Response: 0 = connection/session closed successfully
    pdu.bhs[2] = 0x00;
    pdu.set_itt(itt);
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    pdu
}

/// Build an R2T (Ready to Transfer) PDU.
///
/// Solicits Data-Out PDUs from the initiator for write commands.
pub fn build_r2t(
    lun: u64,
    itt: u32,
    ttt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
    r2t_sn: u32,
    buffer_offset: u32,
    desired_data_transfer_length: u32,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_R2T);
    pdu.set_flags(0x80); // F=1
    pdu.set_lun(lun);
    pdu.set_itt(itt);
    pdu.set_ttt(ttt);
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    pdu.set_r2t_sn(r2t_sn);
    pdu.set_buffer_offset(buffer_offset);
    pdu.set_desired_data_transfer_length(desired_data_transfer_length);
    pdu
}

/// Build a Text Response PDU.
pub fn build_text_response(
    itt: u32,
    ttt: u32,
    stat_sn: u32,
    exp_cmd_sn: u32,
    max_cmd_sn: u32,
    data: Vec<u8>,
) -> Pdu {
    let mut pdu = Pdu::new();
    pdu.set_opcode(OPCODE_TEXT_RESP);
    pdu.set_flags(0x80); // F=1
    pdu.set_data_segment_length(data.len());
    pdu.set_itt(itt);
    pdu.set_ttt(ttt);
    pdu.set_stat_sn(stat_sn);
    pdu.set_exp_cmd_sn(exp_cmd_sn);
    pdu.set_max_cmd_sn(max_cmd_sn);
    pdu.data = data;
    pdu
}
