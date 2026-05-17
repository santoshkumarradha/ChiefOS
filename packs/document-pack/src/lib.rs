//! Document pack for the Platform MVP demo.
//!
//! This pack intentionally uses only the public `chief-sdk` surface.

use chief_sdk::prelude::*;
use serde_json::{json, Value};

pub struct DocumentAgent;

#[async_trait::async_trait]
impl Agent for DocumentAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        let nodes = ctx
            .memory()
            .query(json!({
                "work_object_id": "acme-follow-up",
                "types": ["artifact", "file"]
            }))
            .await?;

        let work_uri = find_work_uri(&nodes)?;
        let contract_uri = find_node_uri_by_kind(&nodes, "fixture_file")?;

        for (idx, obligation) in [
            "Deliver the pilot workspace by 2026-06-03.",
            "Ask Acme to provide a security contact before kickoff.",
            "Flag Net 60 payment terms as a follow-up risk.",
        ]
        .iter()
        .enumerate()
        {
            ctx.memory()
                .put_node(json!({
                    "node_type": "finding",
                    "horizon": "medium",
                    "source": "pack:document-pack/document-agent",
                    "body": {
                        "kind": "obligation",
                        "work_object_id": "acme-follow-up",
                        "work_uri": work_uri,
                        "source_refs": [contract_uri],
                        "ordinal": idx + 1,
                        "content": obligation
                    }
                }))
                .await?;
        }

        Ok(())
    }
}

fn find_work_uri(nodes: &[Value]) -> Result<String> {
    nodes
        .iter()
        .find(|node| node["body"]["kind"] == "work_object")
        .and_then(|node| node["uri"].as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| SdkError::Connector("work object not found".into()))
}

fn find_node_uri_by_kind(nodes: &[Value], kind: &str) -> Result<String> {
    nodes
        .iter()
        .find(|node| node["body"]["kind"] == kind)
        .and_then(|node| node["uri"].as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| SdkError::Connector(format!("{kind} node not found")))
}
