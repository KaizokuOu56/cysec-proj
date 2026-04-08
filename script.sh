#!/bin/bash

# File Integrity Monitor (FIM) Execution Script
# This script loads environment variables and runs the FIM binary

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Load .env file
if [ -f "$SCRIPT_DIR/.env" ]; then
    set -a
    source "$SCRIPT_DIR/.env"
    set +a
else
    echo "[ERROR] .env file not found in $SCRIPT_DIR"
    exit 1
fi

# Check if binary exists
BINARY="$SCRIPT_DIR/target/release/fim"
if [ ! -f "$BINARY" ]; then
    echo "[ERROR] Binary not found at $BINARY"
    echo "[INFO] Building FIM..."
    cd "$SCRIPT_DIR"
    cargo build --release
fi

# Run FIM with the specified directory
# Default to /etc if no argument provided
SCAN_DIR="${1:-.}"
LOG_FILE="/var/log/fim.log"

# Create log file if it doesn't exist (may need sudo)
if [ ! -f "$LOG_FILE" ] 2>/dev/null; then
    LOG_FILE="/tmp/fim-$(date +%s).log"
fi

echo "[INFO] Starting FIM scan at $(date)"
"$BINARY" "$SCAN_DIR" --log "$LOG_FILE"
echo "[INFO] FIM scan completed at $(date)"
