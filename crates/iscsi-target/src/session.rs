//! iSCSI session tracking for the admin dashboard.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

/// Information about an active iSCSI session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub initiator_name: String,
    pub tsih: u16,
    pub peer_addr: String,
    pub connected_since: String,
    pub active_commands: u32,
}

/// Thread-safe registry of active iSCSI sessions.
pub struct SessionRegistry {
    sessions: Mutex<HashMap<u16, SessionInfo>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, tsih: u16, info: SessionInfo) {
        self.sessions.lock().unwrap().insert(tsih, info);
    }

    pub fn unregister(&self, tsih: u16) {
        self.sessions.lock().unwrap().remove(&tsih);
    }

    pub fn snapshot(&self) -> Vec<SessionInfo> {
        self.sessions.lock().unwrap().values().cloned().collect()
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
