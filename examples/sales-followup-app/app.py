#!/usr/bin/env python3
"""Tiny external Chief app used by the POC 2 E2E gate.

The app intentionally uses only HTTP and Python's standard library. It must not
depend on Chief kernel crates or SDK internals.
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


def main() -> int:
    base_url = os.environ.get("CHIEF_BASE_URL", "http://127.0.0.1:18083").rstrip("/")
    work_id = os.environ.get("CHIEF_WORK_ID", "acme-follow-up")
    principal = os.environ.get("CHIEF_APP_PRINCIPAL", "app:sales-followup")

    work = request_json("GET", f"{base_url}/v1/work/{work_id}", principal)
    source_refs = [ref["uri"] for ref in work.get("source_refs", []) if ref.get("uri")]
    contribution_count = len(work.get("contributions", []))

    payload = {
        "node_type": "finding",
        "kind": "next_best_action",
        "title": "Call Acme before sending the draft",
        "summary": (
            "The external sales-followup app recommends a short call before "
            "shipping the drafted reply."
        ),
        "authority_state": "handled",
        "source_refs": source_refs[:3],
        "body": {
            "app": "sales-followup-app",
            "recommendation": "call_first",
            "reason": "The Work Object combines contract, calendar, and prior-email context.",
            "input_work_object_uri": work["uri"],
            "observed_contribution_count": contribution_count,
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
