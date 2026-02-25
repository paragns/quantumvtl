//! iSCSI session tracking for the admin dashboard.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::scsi_log::ScsiCommandLog;

/// Information about an active iSCSI session (legacy flat view).
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub initiator_name: String,
    pub tsih: u16,
    pub peer_addr: String,
    pub connected_since: String,
    pub active_commands: u32,
}

/// Per-connection traffic counters (atomically updated from the I/O path).
pub struct ConnectionStats {
    pub rx_commands: AtomicU64,
    pub rx_bytes: AtomicU64,
    pub tx_commands: AtomicU64,
    pub tx_bytes: AtomicU64,
    pub active_commands: AtomicU32,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self {
            rx_commands: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_commands: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            active_commands: AtomicU32::new(0),
        }
    }
}

/// A single iSCSI connection within a session.
pub struct ConnectionEntry {
    pub cid: u16,
    pub peer_addr: String,
    pub connected_since: String,
    pub stats: Arc<ConnectionStats>,
    pub scsi_log: Arc<ScsiCommandLog>,
}

/// Internal session entry in the registry.
struct SessionEntry {
    initiator_name: String,
    tsih: u16,
    connections: Vec<ConnectionEntry>,
}

/// Snapshot of a single connection (for API responses).
#[derive(Clone)]
pub struct ConnectionSnapshot {
    pub cid: u16,
    pub peer_addr: String,
    pub connected_since: String,
    pub rx_commands: u64,
    pub rx_bytes: u64,
    pub tx_commands: u64,
    pub tx_bytes: u64,
    pub active_commands: u32,
    pub scsi_log: Arc<ScsiCommandLog>,
}

/// Snapshot of a session with enriched connection data.
#[derive(Clone)]
pub struct SessionSnapshot {
    pub initiator_name: String,
    pub tsih: u16,
    pub connections: Vec<ConnectionSnapshot>,
}

/// Thread-safe registry of active iSCSI sessions.
#[derive(Default)]
pub struct SessionRegistry {
    sessions: Mutex<HashMap<u16, SessionEntry>>,
    next_cid: AtomicU16,
}

use std::sync::atomic::AtomicU16;

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_cid: AtomicU16::new(1),
        }
    }

    /// Register a new session (called at login completion).
    pub fn register(&self, tsih: u16, info: SessionInfo) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(
            tsih,
            SessionEntry {
                initiator_name: info.initiator_name,
                tsih,
                connections: Vec::new(),
            },
        );
    }

    /// Register a connection within an existing session.
    /// Returns (Arc<ConnectionStats>, Arc<ScsiCommandLog>) for the caller to use.
    pub fn register_connection(
        &self,
        tsih: u16,
        peer_addr: String,
        connected_since: String,
        scsi_log: Arc<ScsiCommandLog>,
    ) -> Arc<ConnectionStats> {
        let cid = self.next_cid.fetch_add(1, Ordering::Relaxed);
        let stats = Arc::new(ConnectionStats::new());
        let entry = ConnectionEntry {
            cid,
            peer_addr,
            connected_since,
            stats: stats.clone(),
            scsi_log,
        };
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&tsih) {
            session.connections.push(entry);
        }
        stats
    }

    pub fn unregister(&self, tsih: u16) {
        self.sessions.lock().unwrap().remove(&tsih);
    }

    /// Return a flat list of SessionInfo for backward compatibility.
    pub fn snapshot(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .map(|s| {
                let active: u32 = s
                    .connections
                    .iter()
                    .map(|c| c.stats.active_commands.load(Ordering::Relaxed))
                    .sum();
                let peer_addr = s
                    .connections
                    .first()
                    .map(|c| c.peer_addr.clone())
                    .unwrap_or_default();
                let connected_since = s
                    .connections
                    .first()
                    .map(|c| c.connected_since.clone())
                    .unwrap_or_default();
                SessionInfo {
                    initiator_name: s.initiator_name.clone(),
                    tsih: s.tsih,
                    peer_addr,
                    connected_since,
                    active_commands: active,
                }
            })
            .collect()
    }

    /// Return enriched session snapshots with per-connection stats.
    pub fn session_snapshots(&self) -> Vec<SessionSnapshot> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .map(|s| SessionSnapshot {
                initiator_name: s.initiator_name.clone(),
                tsih: s.tsih,
                connections: s
                    .connections
                    .iter()
                    .map(|c| ConnectionSnapshot {
                        cid: c.cid,
                        peer_addr: c.peer_addr.clone(),
                        connected_since: c.connected_since.clone(),
                        rx_commands: c.stats.rx_commands.load(Ordering::Relaxed),
                        rx_bytes: c.stats.rx_bytes.load(Ordering::Relaxed),
                        tx_commands: c.stats.tx_commands.load(Ordering::Relaxed),
                        tx_bytes: c.stats.tx_bytes.load(Ordering::Relaxed),
                        active_commands: c.stats.active_commands.load(Ordering::Relaxed),
                        scsi_log: c.scsi_log.clone(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// Find a SCSI log entry by TSIH and sequence number.
    pub fn find_scsi_log_entry(
        &self,
        tsih: u16,
        seq: u64,
    ) -> Option<crate::scsi_log::ScsiLogEntry> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&tsih) {
            for conn in &session.connections {
                if let Some(entry) = conn.scsi_log.get_by_seq(seq) {
                    return Some(entry);
                }
            }
        }
        None
    }
}

/// RAII guard that unregisters a session on drop.
pub(crate) struct SessionGuard {
    registry: Arc<SessionRegistry>,
    tsih: Option<u16>,
}

impl SessionGuard {
    pub(crate) fn new(registry: Arc<SessionRegistry>) -> Self {
        Self {
            registry,
            tsih: None,
        }
    }

    pub(crate) fn set_tsih(&mut self, tsih: u16) {
        self.tsih = Some(tsih);
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if let Some(tsih) = self.tsih {
            self.registry.unregister(tsih);
        }
    }
}
