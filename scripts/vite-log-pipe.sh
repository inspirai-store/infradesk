#!/usr/bin/env bash
#
# Vite Log Pipe Script
#
# This script reads Vite dev server output from stdin and sends it to the
# backend log aggregation service while also printing to stdout.
#
# Usage:
#   pnpm dev 2>&1 | ./scripts/vite-log-pipe.sh
#

LOG_API_URL="http://127.0.0.1:12420/api/logs"

# Function to send a log entry
send_log() {
    local level="$1"
    local message="$2"

    # Escape special characters in message for JSON
    message=$(echo "$message" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed 's/\t/\\t/g')

    curl -s -X POST "$LOG_API_URL" \
        -H "Content-Type: application/json" \
        -d "{\"source\":\"vite\",\"level\":\"$level\",\"message\":\"$message\"}" \
        > /dev/null 2>&1 &
}

# Function to determine log level from message content
get_level() {
    local line="$1"

    if echo "$line" | grep -qiE "(error|failed|exception)"; then
        echo "error"
    elif echo "$line" | grep -qiE "(warn|warning)"; then
        echo "warn"
    elif echo "$line" | grep -qiE "(debug)"; then
        echo "debug"
    else
        echo "info"
    fi
}

# Main loop: read from stdin, send to API, and print to stdout
while IFS= read -r line || [[ -n "$line" ]]; do
    # Print to stdout
    echo "$line"

    # Skip empty lines
    if [[ -z "$line" ]]; then
        continue
    fi

    # Determine log level
    level=$(get_level "$line")

    # Send to log API (in background to not block)
    send_log "$level" "$line"
done
