use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::ChiefMemError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    DerivedFrom,
    RepliedTo,
    DependsOn,
    RefersTo,
    AuthoredBy,
    Cites,
    Updates,
    Contradicts,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DerivedFrom => "derived-from",
            Self::RepliedTo => "replied-to",
            Self::DependsOn => "depends-on",
            Self::RefersTo => "refers-to",
            Self::AuthoredBy => "authored-by",
            Self::Cites => "cites",
            Self::Updates => "updates",
            Self::Contradicts => "contradicts",
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EdgeKind {
    type Err = ChiefMemError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "derived-from" => Ok(Self::DerivedFrom),
            "replied-to" => Ok(Self::RepliedTo),
            "depends-on" => Ok(Self::DependsOn),
            "refers-to" => Ok(Self::RefersTo),
            "authored-by" => Ok(Self::AuthoredBy),
            "cites" => Ok(Self::Cites),
            "updates" => Ok(Self::Updates),
            "contradicts" => Ok(Self::Contradicts),
            _ => Err(ChiefMemError::InvalidEnum {
                kind: "edge kind",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub weight: f64,
    pub created_by: String,
}
