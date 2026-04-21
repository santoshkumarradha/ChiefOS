//! Pane definitions — declarative UI surfaces.

use serde::{Deserialize, Serialize};

/// Pane properties passed from the OS to pane render code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneProps {
    pub mem_uri: String,
    pub surface: String,
    pub region: u32,
}

/// Pane descriptor: what a pack's pane surface looks like.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneDescriptor {
    pub id: String,
    pub pack_id: String,
    pub surface: String,
    pub region: u32,
    pub title: String,
    pub description: String,
    pub component: String,
}

impl PaneDescriptor {
    pub fn new(
        id: impl Into<String>,
        pack_id: impl Into<String>,
        surface: impl Into<String>,
        region: u32,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            pack_id: pack_id.into(),
            surface: surface.into(),
            region,
            title: title.into(),
            description: String::new(),
            component: String::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = component.into();
        self
    }
}
