//! Pack manifest — declared metadata and grants.

use crate::grant::Grant;
use serde::{Deserialize, Serialize};

/// Pack manifest — what the OS enforces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub grants: Vec<Grant>,
    pub agents: Vec<String>,
    pub panes: Vec<String>,
    pub signature: String,
}

impl PackManifest {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            grants: Vec::new(),
            agents: Vec::new(),
            panes: Vec::new(),
            signature: signature.into(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_grants(mut self, grants: Vec<Grant>) -> Self {
        self.grants = grants;
        self
    }

    pub fn with_agents(mut self, agents: Vec<String>) -> Self {
        self.agents = agents;
        self
    }

    pub fn with_panes(mut self, panes: Vec<String>) -> Self {
        self.panes = panes;
        self
    }
}
