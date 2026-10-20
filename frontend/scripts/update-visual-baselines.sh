#!/usr/bin/env bash
# update-visual-baselines.sh
# Regenerates Playwright screenshot baselines locally and commits them.
# Run this after intentional UI changes to approve the new visuals.
#
# Usage:
#   ./scripts/update-visual-baselines.sh              # all projects
#   ./scripts/update-visual-baselines.sh chromium     # single project
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$(dirname "$SCRIPT_DIR")"

cd "$FRONTEND_DIR"

PROJECT="${1:-}"

if [[ -n "$PROJECT" ]]; then
  echo "Updating baselines for project: visual-$PROJECT"
  npx playwright test --project="visual-$PROJECT" --update-snapshots
else
  echo "Updating baselines for all visual projects..."
  npm run visual:update
fi

echo ""
echo "Baselines updated in e2e/__snapshots__/"
echo "Review the changes with: git diff --stat e2e/__snapshots__/"
echo "Then commit: git add e2e/__snapshots__ && git commit -m 'chore: update visual baselines'"
