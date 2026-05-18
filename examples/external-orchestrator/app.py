#!/usr/bin/env python3
"""POC 4 external orchestrator example.

This process owns planning and "agent" selection locally. Chief is used only as
the OS substrate: Work Object read, contribution write, provenance, Ceremony.
"""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request


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


def plan_agents(work: dict) -> dict:
    return {
        "orchestrator": "external-orchestrator",
        "selected_agents": ["contract-context-agent", "reply-drafter-agent"],
        "steps": [
            "review source_refs and current pack contributions",
            "draft a reply action that must go through Chief Ceremony",
        ],
        "observed_contributions": len(work.get("contributions", [])),
    }


def contract_context_agent(work: dict) -> dict:
    return {
        "agent": "contract-context-agent",
        "finding": "Clause 4 and the Tuesday follow-up window need confirmation before send.",
        "source_count": len(work.get("source_refs", [])),
    }


def reply_drafter_agent(context: dict) -> dict:
    return {
        "to": "maya@acme.example",
        "subject": "Acme follow-up",
        "body": (
            "Thanks for the discussion. I want to confirm clause 4 and the "
            "Tuesday follow-up window before we finalize the next step."
        ),
        "based_on": context["agent"],
    }


def main() -> int:
    base_url = os.environ.get("CHIEF_BASE_URL", "http://127.0.0.1:18086").rstrip("/")
    work_id = os.environ.get("CHIEF_WORK_ID", "acme-follow-up")
    principal = os.environ.get("CHIEF_APP_PRINCIPAL", "app:external-orchestrator")

    work = request_json("GET", f"{base_url}/v1/work/{work_id}", principal)
    source_refs = [ref["uri"] for ref in work.get("source_refs", []) if ref.get("uri")]
    plan = plan_agents(work)
    context = contract_context_agent(work)
    draft = reply_drafter_agent(context)

    payload = {
        "node_type": "artifact",
        "kind": "orchestrated_followup_plan",
        "title": "External orchestrator plan for Acme",
        "summary": "An external orchestrator selected agents and proposed a send action.",
        "source_refs": source_refs[:3],
        "body": {
            "plan": plan,
            "context": context,
            "draft_action": "email.send",
            "payload": draft,
        },
        "ceremony": {
            "title": "Send orchestrated Acme follow-up",
            "summary": "external-orchestrator needs Ceremony before sending externally.",
            "category": "email.send",
            "payload": draft,
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
