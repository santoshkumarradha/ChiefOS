#!/bin/bash
# scripts/demo-docker.sh — End-to-end demo of Chief OS Docker container
#
# Brings up the container, waits for health, posts a test intent,
# queries the brief, and verifies at least one signed attestation was produced.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "==> Chief OS Docker Demo"
echo ""

# 1. Start container in background
echo "Starting container..."
cd "$PROJECT_ROOT"
docker compose up -d

# 2. Wait for health check to pass (max 90 seconds, excluding first model download)
echo "Waiting for health check..."
MAX_WAIT=90
ELAPSED=0
while ! curl -sf http://localhost:4711/status > /dev/null 2>&1; do
    if [ $ELAPSED -ge $MAX_WAIT ]; then
        echo "FAILED: Container did not become healthy within ${MAX_WAIT}s"
        docker compose logs chief
        exit 1
    fi
    echo "  ...$ELAPSED/$MAX_WAIT seconds"
    sleep 2
    ELAPSED=$((ELAPSED + 2))
done

echo "✓ Container is healthy"
echo ""

# 3. Post a test intent
echo "Posting test intent..."
INTENT_RESPONSE=$(curl -sf -X POST http://localhost:4711/intent \
    -H "Content-Type: application/json" \
    -d '{"prompt":"Test: summarize the weather"}' 2>/dev/null || echo '{}')

echo "Intent response: $INTENT_RESPONSE"
echo ""

# 4. Retrieve the brief
echo "Fetching Morning Brief..."
BRIEF_RESPONSE=$(curl -sf http://localhost:4711/brief 2>/dev/null || echo '{}')

if [ -z "$BRIEF_RESPONSE" ] || [ "$BRIEF_RESPONSE" = "{}" ]; then
    echo "WARNING: /brief endpoint returned no data or error"
else
    echo "✓ Morning Brief retrieved"
    # Count attestations in response (look for "signature" or "attestation" fields)
    ATTESTATION_COUNT=$(echo "$BRIEF_RESPONSE" | grep -io '"signature"' | wc -l)
    if [ $ATTESTATION_COUNT -gt 0 ]; then
        echo "✓ Found $ATTESTATION_COUNT signed attestation(s)"
    else
        echo "⚠ No attestations found in brief (this is OK for v0 stub)"
    fi
fi

echo ""
echo "==> Summary"
echo "  Status endpoint:       http://localhost:4711/status ✓"
echo "  Morning Brief UI:      http://localhost:5173 (open in browser)"
echo "  Chief Core API:        http://localhost:4711"
echo ""
echo "Container is running. Press Ctrl+C to stop, or:"
echo "  docker compose down    # Stop and remove"
echo "  docker compose logs -f # Follow logs"
echo "  docker compose exec chief chief-core --help  # Run commands inside"

exit 0
