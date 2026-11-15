pub mod cli;
pub mod mcp;
pub mod protocol;
pub mod scripts;
pub mod server;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;

pub use protocol::AgentResultEnvelope;
pub use scripts::AgentScripts;

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_request_id() -> u64 {
    NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default)]
pub struct PendingAgentRequests {
    pending: std::sync::Mutex<std::collections::HashMap<u64, mpsc::Sender<AgentResultEnvelope>>>,
}

impl PendingAgentRequests {
    pub fn new() -> Self {
        Self {
            pending: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn register(&self, req_id: u64, tx: mpsc::Sender<AgentResultEnvelope>) {
        if let Ok(mut map) = self.pending.lock() {
            map.insert(req_id, tx);
        }
    }

    pub fn complete(&self, req_id: u64, result: AgentResultEnvelope) -> bool {
        if let Ok(mut map) = self.pending.lock() {
            if let Some(tx) = map.remove(&req_id) {
                let _ = tx.send(result);
                return true;
            }
        }
        false
    }
}
