#!/usr/bin/env bash
set -euo pipefail

# fetch-model.sh: Download Qwen2.5 3B Instruct Q4_K_M GGUF from Hugging Face
# Idempotent: skips if already present and SHA256 matches.
# Usage: bash scripts/fetch-model.sh [--force]

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Model config
MODEL_NAME="qwen2.5-3b-instruct-q4_k_m"
MODEL_URL="${MODEL_URL:-https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf}"
# SHA256: Must be set or discovered from HF repo. Placeholder for now.
EXPECTED_SHA256="${MODEL_SHA256:-d41d8cd98f00b204e9800998ecf8427e}" # Placeholder: update with actual

# Destination
CHIEF_MODEL_DIR="${CHIEF_MODEL_DIR:-$HOME/.cache/chief-os/models}"
MODEL_FILE="$CHIEF_MODEL_DIR/$MODEL_NAME.gguf"

FORCE="${1:-}"

echo "Model: $MODEL_NAME"
echo "Destination: $MODEL_FILE"
echo ""

# Check if already exists and matches SHA256
if [ -f "$MODEL_FILE" ] && [ "$FORCE" != "--force" ]; then
    ACTUAL_SHA256="$(shasum -a 256 "$MODEL_FILE" | awk '{print $1}')"
    if [ "$ACTUAL_SHA256" = "$EXPECTED_SHA256" ]; then
        echo "✓ Model already exists and SHA256 matches."
        echo "  File: $MODEL_FILE"
        echo "  SHA256: $ACTUAL_SHA256"
        FILE_SIZE="$(stat -f%z "$MODEL_FILE" 2>/dev/null || stat -c%s "$MODEL_FILE" 2>/dev/null || echo '?')"
        echo "  Size: $FILE_SIZE bytes"
        exit 0
    else
        echo "⚠ Model exists but SHA256 mismatch (corrupted?)."
        echo "  Expected: $EXPECTED_SHA256"
        echo "  Actual:   $ACTUAL_SHA256"
        echo "  Re-downloading with --force..."
    fi
fi

# Create directory
mkdir -p "$CHIEF_MODEL_DIR"

# Download
echo "Downloading from: $MODEL_URL"
echo "This may take several minutes (~2 GB)..."

if ! command -v curl &> /dev/null; then
    echo "✗ curl not found. Please install curl to download models."
    exit 1
fi

# Use curl to download with progress
TEMP_FILE="$MODEL_FILE.tmp"
rm -f "$TEMP_FILE"

if curl -L --show-error --progress-bar "$MODEL_URL" -o "$TEMP_FILE"; then
    # Verify SHA256
    ACTUAL_SHA256="$(shasum -a 256 "$TEMP_FILE" | awk '{print $1}')"
    if [ "$ACTUAL_SHA256" = "$EXPECTED_SHA256" ]; then
        mv "$TEMP_FILE" "$MODEL_FILE"
        echo ""
        echo "✓ Download successful and SHA256 verified."
        FILE_SIZE="$(stat -f%z "$MODEL_FILE" 2>/dev/null || stat -c%s "$MODEL_FILE" 2>/dev/null || echo '?')"
        echo "  File: $MODEL_FILE"
        echo "  Size: $FILE_SIZE bytes"
        echo "  SHA256: $ACTUAL_SHA256"
        exit 0
    else
        rm -f "$TEMP_FILE"
        echo "✗ SHA256 mismatch after download (corrupted or wrong URL?)."
        echo "  Expected: $EXPECTED_SHA256"
        echo "  Actual:   $ACTUAL_SHA256"
        exit 1
    fi
else
    rm -f "$TEMP_FILE"
    echo "✗ Download failed. Check your internet connection and the URL."
    exit 1
fi
