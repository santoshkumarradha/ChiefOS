//! Email pack for the Platform MVP demo.

use chief_sdk::prelude::*;
use serde_json::json;

pub struct EmailAgent;

#[async_trait::async_trait]
impl Agent for EmailAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        let nodes = ctx
            .memory()
            .query(json!({
                "work_object_id": "acme-follow-up",
                "types": ["artifact", "finding", "email"]
            }))
            .await?;

        let obligations = nodes
            .iter()
            .filter(|node| node["body"]["kind"] == "obligation")
            .count();
        let slots = nodes
            .iter()
            .filter(|node| node["body"]["kind"] == "candidate_slot")
            .count();
        if obligations < 3 || slots < 2 {
            return Err(SdkError::Connector(
                "email-pack requires obligations and candidate slots first".into(),
            ));
        }

        ctx.memory()
            .put_node(json!({
                "node_type": "artifact",
                "horizon": "medium",
                "source": "pack:email-pack/email-agent",
                "body": {
                    "kind": "draft_reply",
                    "work_object_id": "acme-follow-up",
                    "subject": "Re: Acme pilot kickoff",
                    "recipient": "maya@acme.example",
                    "body": "Thanks Maya — confirming the June 3 kickoff hold. Could you send the security contact before kickoff? I also want to flag the Net 60 payment term so we can confirm it before invoice setup.",
                    "requires_ceremony": true
                }
            }))
            .await?;

        Ok(())
    }
}
