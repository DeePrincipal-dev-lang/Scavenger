#!/bin/bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $0 [distribution-id] [paths]

Examples:
  $0 E1234567890ABC "/*"
  DISTRIBUTION_ID=E1234567890ABC $0 "/index.html /app.js"

If distribution id is not passed as the first argument, the script reads DISTRIBUTION_ID from the environment.
EOF
  exit 1
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
  usage
fi

if [[ "${1:-}" =~ ^E[A-Z0-9]{10,}$ ]]; then
  DISTRIBUTION_ID="$1"
  shift
else
  DISTRIBUTION_ID="${DISTRIBUTION_ID:-}"
fi

if [[ -z "$DISTRIBUTION_ID" ]]; then
  echo "Error: CloudFront distribution ID is required." >&2
  usage
fi

PATHS="${1:-/*}"

command -v aws >/dev/null 2>&1 || {
  echo "Error: AWS CLI is required to run this script." >&2
  exit 1
}

echo "Invalidating CloudFront distribution: $DISTRIBUTION_ID"
echo "Paths: $PATHS"

# Create invalidation
INVALIDATION_ID=$(aws cloudfront create-invalidation \
  --distribution-id "$DISTRIBUTION_ID" \
  --paths $PATHS \
  --query 'Invalidation.Id' \
  --output text)

echo "Invalidation created: $INVALIDATION_ID"

# Wait for invalidation to complete
echo "Waiting for invalidation to complete..."
aws cloudfront wait invalidation-completed \
  --distribution-id "$DISTRIBUTION_ID" \
  --id "$INVALIDATION_ID"

echo "Invalidation complete!"
