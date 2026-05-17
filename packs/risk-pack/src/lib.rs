//! Risk pack for the Platform MVP demo.
//!
//! This pack intentionally uses only the public `chief-sdk` surface.

use chief_sdk::prelude::*;
use serde_json::json;

pub struct RiskAgent;

#[async_trait::async_trait]
impl Agent for RiskAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        let nodes = ctx
            .memory()
            .query(json!({
                "work_object_id": "acme-follow-up",
                "types": ["artifact", "finding"]
            }))
            .await?;

        let work_uri = nodes
            .iter()
            .find(|node| node["body"]["kind"] == "work_object")
            .and_then(|node| node["uri"].as_str())
            .ok_or_else(|| SdkError::Connector("work object not found".into()))?;

        let payment_obligation = nodes.iter().find(|node| {
            node["body"]["kind"] == "obligation"
                && node["body"]["content"]
                    .as_str()
                    .map(|text| text.contains("Net 60"))
                    .unwrap_or(false)
        });
        let Some(obligation) = payment_obligation else {
            return Err(SdkError::Connector(
                "risk-pack requires a payment obligation".into(),
            ));
        };

        ctx.memory()
            .put_node(json!({
                "node_type": "finding",
                "horizon": "medium",
                "source": "pack:risk-pack/risk-agent",
                "body": {
                    "kind": "risk",
                    "work_object_id": "acme-follow-up",
                    "work_uri": work_uri,
                    "severity": "medium",
                    "content": "Net 60 payment terms should be confirmed before invoice setup.",
                    "source_refs": [obligation["uri"].as_str().unwrap_or_default()]
                }
            }))
            .await?;

        Ok(())
    }
}
