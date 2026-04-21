//! Inference router with 5-level policy stack (call > agent > pack > category > system).

use crate::backends::{BackendId, ModelBackend};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Policy {
    Backend(BackendId),
    Inherit,
}

pub struct InferenceRouter {
    backends: HashMap<String, Arc<dyn ModelBackend>>,
    system_default: BackendId,
}

impl InferenceRouter {
    pub fn new(system_default: BackendId) -> Self {
        Self {
            backends: HashMap::new(),
            system_default,
        }
    }
    
    pub fn register_backend(&mut self, backend: Arc<dyn ModelBackend>) {
        let id = backend.id().0.clone();
        self.backends.insert(id, backend);
    }
    
    pub fn resolve(
        &self,
        call_override: Option<&BackendId>,
        agent_preference: Option<&BackendId>,
        pack_policy: Option<&BackendId>,
        category_policy: Option<&BackendId>,
    ) -> Result<Arc<dyn ModelBackend>> {
        let selected_id = call_override
            .or(agent_preference)
            .or(pack_policy)
            .or(category_policy)
            .unwrap_or(&self.system_default);
        
        self.get_backend(selected_id)
    }
    
    pub fn get_backend(&self, id: &BackendId) -> Result<Arc<dyn ModelBackend>> {
        self.backends
            .get(&id.0)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("backend not found: {}", id.0))
    }
}
