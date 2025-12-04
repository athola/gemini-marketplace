#!/usr/bin/env bash
set -euo pipefail
SPEC="specs/001-build-a-gemini/contracts/marketplace-openapi.yaml"
if [ ! -f "$SPEC" ]; then
  echo "error: cannot find $SPEC" >&2
  exit 1
fi
if ! command -v npx >/dev/null 2>&1; then
  echo "warning: npx not found; install Node.js to run OpenAPI lint" >&2
  exit 1
fi
npx --yes @redocly/cli lint "$SPEC"
