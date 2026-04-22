//! Depth-capped meta-prompt helper.

use serde_json::{json, Value};

pub const MAX_META_PROMPT_DEPTH: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaPromptDecision {
    Allowed { requested_depth: u8 },
    Denied { reason: String },
}

pub fn evaluate_meta_prompt(current_depth: u8, arguments: &Value) -> MetaPromptDecision {
    let requested_depth = arguments
        .get("depth")
        .and_then(|value| value.as_u64())
        .unwrap_or(u64::from(current_depth) + 1) as u8;

    if requested_depth > MAX_META_PROMPT_DEPTH || current_depth >= MAX_META_PROMPT_DEPTH {
        return MetaPromptDecision::Denied {
            reason: format!("meta.prompt depth cap exceeded: max {MAX_META_PROMPT_DEPTH}"),
        };
    }

    MetaPromptDecision::Allowed { requested_depth }
}

pub fn child_result_json(requested_depth: u8) -> Value {
    json!({
        "status": "completed",
        "child_depth": requested_depth,
        "termination": "Completed"
    })
}
