#!/bin/bash
# Load testing script using wrk

set -euo pipefail

TARGET_URL="${1:-https://api.scavenger.com}"
THREADS="${2:-12}"
CONNECTIONS="${3:-400}"
DURATION="${4:-30s}"

echo "Starting load test..."
echo "URL: $TARGET_URL"
echo "Threads: $THREADS"
echo "Connections: $CONNECTIONS"
echo "Duration: $DURATION"
echo ""

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
  echo "Installing wrk..."
  git clone https://github.com/wg/wrk.git /tmp/wrk
  cd /tmp/wrk
  make
  sudo cp wrk /usr/local/bin/
fi

# Run load test
wrk -t"$THREADS" -c"$CONNECTIONS" -d"$DURATION" \
  -s /dev/stdin "$TARGET_URL" << 'EOF'
request = function()
  wrk.method = "GET"
  wrk.path = "/health"
  return wrk.format(nil)
end

response = function(status, headers, body)
  if status ~= 200 then
    io.write("HTTP " .. status .. "\n")
  end
end
EOF

echo ""
echo "Load test completed"
