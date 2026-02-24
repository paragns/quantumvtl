use tracing::{debug, warn};

use crate::pdu::{self, Pdu};

/// Parse iSCSI text key=value pairs from a data segment.
pub fn parse_kv_pairs(data: &[u8]) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for entry in data.split(|&b| b == 0) {
        let s = match std::str::from_utf8(entry) {
            Ok(s) if !s.is_empty() => s,
            _ => continue,
        };
        if let Some((k, v)) = s.split_once('=') {
            pairs.push((k.to_string(), v.to_string()));
        }
    }
    pairs
}

/// Serialize key=value pairs into a null-terminated data segment.
pub fn serialize_kv_pairs(pairs: &[(&str, &str)]) -> Vec<u8> {
    let mut data = Vec::new();
    for (k, v) in pairs {
        data.extend_from_slice(k.as_bytes());
        data.push(b'=');
        data.extend_from_slice(v.as_bytes());
        data.push(0);
    }
    data
}

/// Login negotiation state.
pub struct LoginNegotiator {
    pub target_name: String,
    pub tsih: u16,
    pub initiator_name: Option<String>,
    next_tsih: u16,
}

impl LoginNegotiator {
    pub fn new(target_name: String) -> Self {
        Self {
            target_name,
            tsih: 0,
            initiator_name: None,
            next_tsih: 1,
        }
    }

    /// Handle a login request PDU. Returns a login response PDU.
    pub fn handle_login(&mut self, req: &Pdu, stat_sn: u32, exp_cmd_sn: u32) -> Pdu {
        let csg = req.login_csg();
        let nsg = req.login_nsg();
        let transit = req.login_transit();

        let kv = parse_kv_pairs(&req.data);
        debug!(csg, nsg, transit, ?kv, "login request");

        match csg {
            // Security negotiation stage
            0 => self.handle_security_stage(req, &kv, nsg, transit, stat_sn, exp_cmd_sn),
            // Login operational negotiation stage
            1 => self.handle_operational_stage(req, nsg, transit, stat_sn, exp_cmd_sn),
            _ => {
                warn!(csg, "unexpected login CSG");
                pdu::build_login_response(
                    req,
                    stat_sn,
                    exp_cmd_sn,
                    exp_cmd_sn,
                    csg,
                    nsg,
                    false,
                    0,
                    2,
                    0, // Initiator Error
                    Vec::new(),
                )
            }
        }
    }

    fn handle_security_stage(
        &mut self,
        req: &Pdu,
        kv: &[(String, String)],
        nsg: u8,
        transit: bool,
        stat_sn: u32,
        exp_cmd_sn: u32,
    ) -> Pdu {
        // Validate target name and capture initiator name
        for (k, v) in kv {
            if k == "TargetName" && v != &self.target_name {
                warn!(target = %v, expected = %self.target_name, "target name mismatch");
                return pdu::build_login_response(
                    req,
                    stat_sn,
                    exp_cmd_sn,
                    exp_cmd_sn,
                    0,
                    0,
                    false,
                    0,
                    2,
                    3, // Not Found
                    Vec::new(),
                );
            }
            if k == "InitiatorName" {
                self.initiator_name = Some(v.clone());
            }
        }

        // Assign TSIH for new sessions
        if req.tsih() == 0 {
            self.tsih = self.next_tsih;
            self.next_tsih = self.next_tsih.wrapping_add(1);
            if self.next_tsih == 0 {
                self.next_tsih = 1;
            }
        }

        let response_pairs: Vec<(&str, &str)> =
            vec![("TargetPortalGroupTag", "1"), ("AuthMethod", "None")];
        let data = serialize_kv_pairs(&response_pairs);

        // If transit, move to the next stage
        let (resp_csg, resp_nsg, resp_transit) = if transit {
            (0, nsg, true)
        } else {
            (0, 0, false)
        };

        pdu::build_login_response(
            req,
            stat_sn,
            exp_cmd_sn,
            exp_cmd_sn,
            resp_csg,
            resp_nsg,
            resp_transit,
            self.tsih,
            0,
            0, // Success
            data,
        )
    }

    fn handle_operational_stage(
        &mut self,
        req: &Pdu,
        nsg: u8,
        transit: bool,
        stat_sn: u32,
        exp_cmd_sn: u32,
    ) -> Pdu {
        let response_pairs: Vec<(&str, &str)> = vec![
            ("HeaderDigest", "None"),
            ("DataDigest", "None"),
            ("MaxRecvDataSegmentLength", "262144"),
            ("InitialR2T", "Yes"),
            ("ImmediateData", "Yes"),
            ("MaxBurstLength", "262144"),
            ("FirstBurstLength", "65536"),
            ("MaxConnections", "1"),
            ("MaxOutstandingR2T", "1"),
            ("ErrorRecoveryLevel", "0"),
            ("DefaultTime2Wait", "2"),
            ("DefaultTime2Retain", "0"),
        ];
        let data = serialize_kv_pairs(&response_pairs);

        let (resp_csg, resp_nsg, resp_transit) = if transit {
            (1, nsg, true)
        } else {
            (1, 0, false)
        };

        pdu::build_login_response(
            req,
            stat_sn,
            exp_cmd_sn,
            exp_cmd_sn,
            resp_csg,
            resp_nsg,
            resp_transit,
            self.tsih,
            0,
            0, // Success
            data,
        )
    }
}
