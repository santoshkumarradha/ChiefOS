use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::ChiefMemError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    Email,
    File,
    Person,
    Event,
    Decision,
    Artifact,
    Thought,
    Finding,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::File => "file",
            Self::Person => "person",
            Self::Event => "event",
            Self::Decision => "decision",
            Self::Artifact => "artifact",
            Self::Thought => "thought",
            Self::Finding => "finding",
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeType {
    type Err = ChiefMemError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "email" => Ok(Self::Email),
            "file" => Ok(Self::File),
            "person" => Ok(Self::Person),
            "event" => Ok(Self::Event),
            "decision" => Ok(Self::Decision),
            "artifact" => Ok(Self::Artifact),
            "thought" => Ok(Self::Thought),
            "finding" => Ok(Self::Finding),
            _ => Err(ChiefMemError::InvalidEnum {
                kind: "node type",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Horizon {
    Short,
    Medium,
    Long,
    Open,
}

impl Horizon {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Medium => "medium",
            Self::Long => "long",
            Self::Open => "open",
        }
    }
}

impl fmt::Display for Horizon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Horizon {
    type Err = ChiefMemError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "short" => Ok(Self::Short),
            "medium" => Ok(Self::Medium),
            "long" => Ok(Self::Long),
            "open" => Ok(Self::Open),
            _ => Err(ChiefMemError::InvalidEnum {
                kind: "horizon",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_type: NodeType,
    pub horizon: Horizon,
    pub confidence: f64,
    pub source: String,
    pub body: String,
    pub blob_ref: Option<String>,
    pub meta_json: Option<serde_json::Value>,
}

impl Node {
    pub fn new(
        node_type: NodeType,
        horizon: Horizon,
        source: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            node_type,
            horizon,
            confidence: 1.0,
            source: source.into(),
            body: body.into(),
            blob_ref: None,
            meta_json: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredNode {
    pub uri: String,
    pub content_hash: String,
    #[serde(flatten)]
    pub node: Node,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeScore {
    pub uri: String,
    pub node: Node,
    pub score: f64,
}

/// Format a node URI as mem://<type>/<content-hash>
pub fn format_uri(node_type: NodeType, content_hash: &str) -> String {
    format!("mem://{}/{}", node_type, content_hash)
}
