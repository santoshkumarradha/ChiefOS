//! CapabilityContext — the ONLY object through which a pack touches the kernel.

use crate::connectors::{EventBusConnector, InferenceConnector, MemoryConnector, NetworkConnector};
use std::sync::Arc;

/// The capability context passed to all pack code.
/// This is the gate: every I/O must go through here, and the broker enforces grants.
pub struct CapabilityContext {
    pub(crate) net: Arc<dyn NetworkConnector>,
    pub(crate) mem: Arc<dyn MemoryConnector>,
    pub(crate) llm: Arc<dyn InferenceConnector>,
    pub(crate) events: Arc<dyn EventBusConnector>,
}

impl CapabilityContext {
    pub fn new(
        net: Arc<dyn NetworkConnector>,
        mem: Arc<dyn MemoryConnector>,
        llm: Arc<dyn InferenceConnector>,
        events: Arc<dyn EventBusConnector>,
    ) -> Self {
        Self {
            net,
            mem,
            llm,
            events,
        }
    }

    pub fn net(&self) -> &Arc<dyn NetworkConnector> {
        &self.net
    }

    pub fn memory(&self) -> &Arc<dyn MemoryConnector> {
        &self.mem
    }

    pub fn llm(&self) -> &Arc<dyn InferenceConnector> {
        &self.llm
    }

    pub fn event_bus(&self) -> &Arc<dyn EventBusConnector> {
        &self.events
    }
}

impl Clone for CapabilityContext {
    fn clone(&self) -> Self {
        Self {
            net: Arc::clone(&self.net),
            mem: Arc::clone(&self.mem),
            llm: Arc::clone(&self.llm),
            events: Arc::clone(&self.events),
        }
    }
}
