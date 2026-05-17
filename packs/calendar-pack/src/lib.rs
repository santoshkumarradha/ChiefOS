//! Calendar pack for the Platform MVP demo.

use chief_sdk::prelude::*;
use serde_json::json;

pub struct CalendarAgent;

#[async_trait::async_trait]
impl Agent for CalendarAgent {
    async fn on_tick(&self, ctx: CapabilityContext) -> Result<()> {
        let nodes = ctx
            .memory()
            .query(json!({
                "work_object_id": "acme-follow-up",
                "types": ["artifact", "finding", "event"]
            }))
            .await?;

        let obligation_count = nodes
            .iter()
            .filter(|node| node["body"]["kind"] == "obligation")
            .count();
        if obligation_count < 3 {
            return Err(SdkError::Connector(
                "calendar-pack requires document obligations first".into(),
            ));
        }

        for (idx, slot) in [
            ("2026-06-03T15:00:00Z", "2026-06-03T15:30:00Z"),
            ("2026-06-04T17:00:00Z", "2026-06-04T17:30:00Z"),
        ]
        .iter()
        .enumerate()
        {
            ctx.memory()
                .put_node(json!({
                    "node_type": "finding",
                    "horizon": "medium",
                    "source": "pack:calendar-pack/calendar-agent",
                    "body": {
                        "kind": "candidate_slot",
                        "work_object_id": "acme-follow-up",
                        "ordinal": idx + 1,
                        "start": slot.0,
                        "end": slot.1,
                        "rationale": "Derived from Acme kickoff timing and open calendar fixture."
                    }
                }))
                .await?;
        }

        Ok(())
    }
}
