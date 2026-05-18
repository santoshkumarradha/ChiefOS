#!/usr/bin/env python3
"""AgentField adapter example for Chief.

This is intentionally a user-space adapter. Chief is not importing AgentField or
running its orchestration. The adapter owns the AgentField-shaped workflow and
talks to Chief through the same public HTTP protocol as every other app.
"""

from __future__ import annotations

import json
import os
import pathlib
import sys
import urllib.error
import urllib.request


DEFAULT_SDK_PATH = (
    "/Users/santoshkumarradha/Documents/agentfield/code/labs/"
    "af-research-lab/agentfield-sdk"
)


def request_json(method: str, url: str, principal: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(url, data=data, method=method)
    request.add_header("accept", "application/json")
    request.add_header("x-chief-principal", principal)
    if data is not None:
        request.add_header("content-type", "application/json")

    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as err:
        details = err.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"{method} {url} failed: {err.code} {details}") from err


def detect_agentfield_sdk() -> dict:
    sdk_path = pathlib.Path(os.environ.get("AGENTFIELD_SDK_PATH", DEFAULT_SDK_PATH))
    if sdk_path.exists():
        sys.path.insert(0, str(sdk_path))
    try:
        import agentfield  # type: ignore

        return {
            "available": True,
            "module": getattr(agentfield, "__name__", "agentfield"),
            "path": str(sdk_path),
        }
    except Exception as exc:  # pragma: no cover - diagnostic metadata only
        return {
            "available": False,
            "path": str(sdk_path),
            "error": exc.__class__.__name__,
        }


def agentfield_reasoner(work: dict) -> dict:
    """AgentField-shaped reasoner output.

    In a real AgentField app this function body would sit behind `@app.reasoner`.
    The important Chief boundary is unchanged: the result is submitted through
    Chief HTTP, not through kernel imports.
    """

    return {
        "agentfield_adapter": True,
        "reasoner": "acme_followup_orchestrator",
        "graph": [
            {"node": "inspect_work_object", "uses": "chief_http"},
            {"node": "synthesize_followup_action", "uses": "agentfield_user_space"},
            {"node": "submit_to_chief", "uses": "chief_http"},
        ],
        "observed_sources": len(work.get("source_refs", [])),
        "observed_contributions": len(work.get("contributions", [])),
    }


def main() -> int:
    base_url = os.environ.get("CHIEF_BASE_URL", "http://127.0.0.1:18087").rstrip("/")
    work_id = os.environ.get("CHIEF_WORK_ID", "acme-follow-up")
    principal = os.environ.get("CHIEF_APP_PRINCIPAL", "app:agentfield-adapter")

    sdk = detect_agentfield_sdk()
    work = request_json("GET", f"{base_url}/v1/work/{work_id}", principal)
    source_refs = [ref["uri"] for ref in work.get("source_refs", []) if ref.get("uri")]
    reasoner_output = agentfield_reasoner(work)
    action_payload = {
        "to": "maya@acme.example",
        "subject": "Acme follow-up",
        "body": "AgentField adapter proposes a governed follow-up action via Chief.",
    }

    payload = {
        "node_type": "artifact",
        "kind": "agentfield_orchestrated_plan",
        "title": "AgentField adapter plan for Acme",
        "summary": "AgentField-shaped adapter produced a plan and proposed a governed send.",
        "source_refs": source_refs[:3],
        "body": {
            "sdk": sdk,
            "reasoner_output": reasoner_output,
            "draft_action": "email.send",
            "payload": action_payload,
        },
        "ceremony": {
            "title": "Send AgentField-adapter Acme follow-up",
            "summary": "agentfield-adapter needs Ceremony before sending externally.",
            "category": "email.send",
            "payload": action_payload,
            "trust_context": 3,
        },
    }

    result = request_json(
        "POST",
        f"{base_url}/v1/work/{work_id}/contributions",
        principal,
        payload,
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
