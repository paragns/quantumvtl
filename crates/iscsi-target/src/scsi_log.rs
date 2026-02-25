use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{ScsiDevice, ScsiResult};

/// Maximum entries retained in the ring buffer.
const DEFAULT_CAPACITY: usize = 20;

/// Identifies the type of SCSI device for opcode name resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    MediaChanger,
    TapeDrive,
}

/// A single SCSI command/response exchange record.
#[derive(Debug, Clone, Serialize)]
pub struct ScsiLogEntry {
    pub seq: u64,
    pub timestamp: String,
    pub duration_us: u64,
    pub cdb: Vec<u8>,
    pub opcode: u8,
    pub opcode_name: String,
    pub data_out: Option<Vec<u8>>,
    pub data_out_len: usize,
    pub status: u8,
    pub data_in: Option<Vec<u8>>,
    pub data_in_len: usize,
    pub sense: Vec<u8>,
    pub completed: bool,
}

/// Thread-safe ring buffer of recent SCSI command/response pairs.
pub struct ScsiCommandLog {
    entries: Mutex<VecDeque<ScsiLogEntry>>,
    capacity: usize,
    next_seq: AtomicU64,
    device_type: DeviceType,
}

impl ScsiCommandLog {
    pub fn new(device_type: DeviceType, capacity: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            next_seq: AtomicU64::new(1),
            device_type,
        }
    }

    /// Record a completed SCSI exchange in one shot, returning the assigned sequence number.
    pub fn record(
        &self,
        cdb: &[u8],
        data_out: &[u8],
        result: &ScsiResult,
        duration_us: u64,
    ) -> u64 {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let opcode = cdb.first().copied().unwrap_or(0);
        let omit_payload = is_data_payload_opcode(opcode, self.device_type);

        let entry = ScsiLogEntry {
            seq,
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            duration_us,
            cdb: cdb.to_vec(),
            opcode,
            opcode_name: opcode_name(opcode, self.device_type).to_string(),
            data_out: if omit_payload || data_out.is_empty() {
                None
            } else {
                Some(data_out.to_vec())
            },
            data_out_len: data_out.len(),
            status: result.status,
            data_in: if omit_payload || result.data_in.is_empty() {
                None
            } else {
                Some(result.data_in.clone())
            },
            data_in_len: result.data_in.len(),
            sense: result.sense.clone(),
            completed: true,
        };

        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
        seq
    }

    /// Record the start of a SCSI command (in-progress). Returns the sequence number
    /// to pass to `record_complete()` when the command finishes.
    pub fn record_start(&self, cdb: &[u8], data_out: &[u8]) -> u64 {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let opcode = cdb.first().copied().unwrap_or(0);
        let omit_payload = is_data_payload_opcode(opcode, self.device_type);

        let entry = ScsiLogEntry {
            seq,
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            duration_us: 0,
            cdb: cdb.to_vec(),
            opcode,
            opcode_name: opcode_name(opcode, self.device_type).to_string(),
            data_out: if omit_payload || data_out.is_empty() {
                None
            } else {
                Some(data_out.to_vec())
            },
            data_out_len: data_out.len(),
            status: 0,
            data_in: None,
            data_in_len: 0,
            sense: vec![],
            completed: false,
        };

        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
        seq
    }

    /// Update an in-progress entry with the final result and duration.
    pub fn record_complete(&self, seq: u64, result: &ScsiResult, duration_us: u64) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.iter_mut().find(|e| e.seq == seq) {
            let omit_payload = is_data_payload_opcode(entry.opcode, self.device_type);
            entry.duration_us = duration_us;
            entry.status = result.status;
            entry.data_in = if omit_payload || result.data_in.is_empty() {
                None
            } else {
                Some(result.data_in.clone())
            };
            entry.data_in_len = result.data_in.len();
            entry.sense = result.sense.clone();
            entry.completed = true;
        }
    }

    /// Return the last N entries (most recent last).
    pub fn last_n(&self, n: usize) -> Vec<ScsiLogEntry> {
        let entries = self.entries.lock().unwrap();
        let skip = entries.len().saturating_sub(n);
        entries.iter().skip(skip).cloned().collect()
    }

    /// Find a specific entry by sequence number.
    pub fn get_by_seq(&self, seq: u64) -> Option<ScsiLogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().find(|e| e.seq == seq).cloned()
    }
}

/// SCSI device wrapper that records command/response pairs into a `ScsiCommandLog`.
pub struct TracedDevice {
    inner: Arc<dyn ScsiDevice>,
    log: Arc<ScsiCommandLog>,
    ws_tx: Option<broadcast::Sender<()>>,
}

impl TracedDevice {
    /// Wrap a SCSI device with tracing. Returns the wrapper and a shared handle to the log.
    pub fn new(
        inner: Arc<dyn ScsiDevice>,
        device_type: DeviceType,
        capacity: usize,
        ws_tx: Option<broadcast::Sender<()>>,
    ) -> (Self, Arc<ScsiCommandLog>) {
        let log = Arc::new(ScsiCommandLog::new(device_type, capacity));
        let traced = Self {
            inner,
            log: log.clone(),
            ws_tx,
        };
        (traced, log)
    }

    /// Create with the default capacity of 20.
    pub fn with_defaults(
        inner: Arc<dyn ScsiDevice>,
        device_type: DeviceType,
        ws_tx: Option<broadcast::Sender<()>>,
    ) -> (Self, Arc<ScsiCommandLog>) {
        Self::new(inner, device_type, DEFAULT_CAPACITY, ws_tx)
    }
}

impl ScsiDevice for TracedDevice {
    fn execute_command(&self, cdb: &[u8], data_out: &[u8]) -> ScsiResult {
        let seq = self.log.record_start(cdb, data_out);

        // Notify WS: command arrived (in-progress)
        if let Some(ref tx) = self.ws_tx {
            let _ = tx.send(());
        }

        let start = Instant::now();
        let result = self.inner.execute_command(cdb, data_out);
        let duration_us = start.elapsed().as_micros() as u64;

        self.log.record_complete(seq, &result, duration_us);

        // Notify WS: command completed
        if let Some(ref tx) = self.ws_tx {
            let _ = tx.send(());
        }

        result
    }
}

/// Returns true for opcodes whose bulk data payloads should be omitted from the log.
pub fn is_data_payload_opcode(opcode: u8, device_type: DeviceType) -> bool {
    match device_type {
        DeviceType::TapeDrive => matches!(opcode, 0x08 | 0x0A), // READ(6), WRITE(6)
        DeviceType::MediaChanger => false, // changers don't have bulk data transfer opcodes
    }
}

/// Map an opcode byte to a human-readable SCSI command name.
pub fn opcode_name(opcode: u8, device_type: DeviceType) -> &'static str {
    match device_type {
        DeviceType::MediaChanger => smc_opcode_name(opcode),
        DeviceType::TapeDrive => ssc_opcode_name(opcode),
    }
}

fn smc_opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0x00 => "TEST UNIT READY",
        0x03 => "REQUEST SENSE",
        0x07 => "INITIALIZE ELEMENT STATUS",
        0x12 => "INQUIRY",
        0x15 => "MODE SELECT(6)",
        0x16 => "RESERVE ELEMENT(6)",
        0x17 => "RELEASE ELEMENT(6)",
        0x1A => "MODE SENSE(6)",
        0x1D => "SEND DIAGNOSTIC",
        0x1E => "PREVENT ALLOW MEDIUM REMOVAL",
        0x2B => "POSITION TO ELEMENT",
        0x3B => "WRITE BUFFER",
        0x3C => "READ BUFFER",
        0x4D => "LOG SENSE",
        0x55 => "MODE SELECT(10)",
        0x56 => "RESERVE ELEMENT(10)",
        0x57 => "RELEASE ELEMENT(10)",
        0x5A => "MODE SENSE(10)",
        0x5E => "PERSISTENT RESERVE IN",
        0x5F => "PERSISTENT RESERVE OUT",
        0xA0 => "REPORT LUNS",
        0xA5 => "MOVE MEDIUM",
        0xA6 => "EXCHANGE MEDIUM",
        0xB5 => "REQUEST VOLUME ELEMENT ADDRESS",
        0xB6 => "SEND VOLUME TAG",
        0xB8 => "READ ELEMENT STATUS",
        0xE7 => "INIT ELEMENT STATUS WITH RANGE",
        _ => "UNKNOWN",
    }
}

fn ssc_opcode_name(opcode: u8) -> &'static str {
    match opcode {
        0x00 => "TEST UNIT READY",
        0x01 => "REWIND",
        0x03 => "REQUEST SENSE",
        0x04 => "FORMAT MEDIUM",
        0x05 => "READ BLOCK LIMITS",
        0x08 => "READ(6)",
        0x0A => "WRITE(6)",
        0x0B => "SET CAPACITY",
        0x10 => "WRITE FILEMARKS(6)",
        0x11 => "SPACE(6)",
        0x12 => "INQUIRY",
        0x13 => "VERIFY(6)",
        0x15 => "MODE SELECT(6)",
        0x19 => "ERASE(6)",
        0x1A => "MODE SENSE(6)",
        0x1B => "LOAD UNLOAD",
        0x1E => "PREVENT ALLOW MEDIUM REMOVAL",
        0x2B => "LOCATE(10)",
        0x34 => "READ POSITION",
        0x44 => "REPORT DENSITY SUPPORT",
        0x4C => "LOG SELECT",
        0x4D => "LOG SENSE",
        0x55 => "MODE SELECT(10)",
        0x5A => "MODE SENSE(10)",
        0x5E => "PERSISTENT RESERVE IN",
        0x5F => "PERSISTENT RESERVE OUT",
        0x80 => "WRITE FILEMARKS(16)",
        0x82 => "ALLOW OVERWRITE",
        0x8C => "READ ATTRIBUTE",
        0x8D => "WRITE ATTRIBUTE",
        0x91 => "SPACE(16)",
        0x92 => "LOCATE(16)",
        0xA0 => "REPORT LUNS",
        0xA3 => "MAINTENANCE IN",
        0xA4 => "MAINTENANCE OUT",
        _ => "UNKNOWN",
    }
}

/// Map a SCSI status byte to its name.
pub fn scsi_status_name(status: u8) -> &'static str {
    match status {
        0x00 => "GOOD",
        0x02 => "CHECK CONDITION",
        0x04 => "CONDITION MET",
        0x08 => "BUSY",
        0x18 => "RESERVATION CONFLICT",
        0x28 => "TASK SET FULL",
        0x30 => "ACA ACTIVE",
        0x40 => "TASK ABORTED",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_capacity() {
        let log = ScsiCommandLog::new(DeviceType::MediaChanger, 3);
        let dummy_result = ScsiResult {
            status: 0,
            data_in: vec![],
            sense: vec![],
        };

        for i in 0..5 {
            log.record(&[i], &[], &dummy_result, 100);
        }

        let entries = log.last_n(10);
        assert_eq!(entries.len(), 3);
        // Should have seqs 3, 4, 5 (first two evicted)
        assert_eq!(entries[0].seq, 3);
        assert_eq!(entries[2].seq, 5);
    }

    #[test]
    fn get_by_seq() {
        let log = ScsiCommandLog::new(DeviceType::TapeDrive, 5);
        let dummy_result = ScsiResult {
            status: 0,
            data_in: vec![],
            sense: vec![],
        };

        log.record(&[0x08], &[], &dummy_result, 50);
        log.record(&[0x0A], &[], &dummy_result, 75);

        assert!(log.get_by_seq(1).is_some());
        assert_eq!(log.get_by_seq(1).unwrap().opcode_name, "READ(6)");
        assert_eq!(log.get_by_seq(2).unwrap().opcode_name, "WRITE(6)");
        assert!(log.get_by_seq(99).is_none());
    }

    #[test]
    fn data_payload_omission() {
        let log = ScsiCommandLog::new(DeviceType::TapeDrive, 5);
        let result_with_data = ScsiResult {
            status: 0,
            data_in: vec![1, 2, 3, 4],
            sense: vec![],
        };

        // READ(6) should omit data
        log.record(&[0x08], &[], &result_with_data, 50);
        let entry = log.get_by_seq(1).unwrap();
        assert!(entry.data_in.is_none());
        assert_eq!(entry.data_in_len, 4);

        // INQUIRY should keep data
        log.record(&[0x12], &[], &result_with_data, 50);
        let entry = log.get_by_seq(2).unwrap();
        assert!(entry.data_in.is_some());
        assert_eq!(entry.data_in_len, 4);
    }

    #[test]
    fn opcode_names() {
        assert_eq!(opcode_name(0xA5, DeviceType::MediaChanger), "MOVE MEDIUM");
        assert_eq!(opcode_name(0x08, DeviceType::TapeDrive), "READ(6)");
        assert_eq!(opcode_name(0x12, DeviceType::MediaChanger), "INQUIRY");
        assert_eq!(opcode_name(0xFF, DeviceType::TapeDrive), "UNKNOWN");
    }
}
