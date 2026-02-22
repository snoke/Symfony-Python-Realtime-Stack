#!/usr/bin/env bash
set -euo pipefail

BASE_URL=${BASE_URL:-http://localhost:8180}
WS_URL=${WS_URL:-ws://localhost:8180/ws}
JWT_PRIVATE_KEY_FILE=${JWT_PRIVATE_KEY_FILE:-./scripts/keys/dev_private.pem}
JWT_ALG=${JWT_ALG:-RS256}
JWT_USER_ID=${JWT_USER_ID:-42}
GATEWAY_API_KEY=${GATEWAY_API_KEY:-dev-key}
SKIP_WS=${SKIP_WS:-0}
SKIP_PUBLISH=${SKIP_PUBLISH:-0}

say() { printf "[smoke] %s\n" "$*"; }

say "HTTP /health"
curl -fsS "$BASE_URL/health" | python -c 'import json,sys; obj=json.load(sys.stdin); assert obj.get("ok") is True, obj; print("ok")'

say "HTTP /ready"
curl -fsS "$BASE_URL/ready" | python -c 'import json,sys; obj=json.load(sys.stdin); assert obj.get("ok") is True, obj; print("ok")'

say "HTTP /metrics"
curl -fsS "$BASE_URL/metrics" | grep -q "^ws_connections_total "

say "HTTP /internal/connections"
curl -fsS "$BASE_URL/internal/connections" | python -c 'import json,sys; obj=json.load(sys.stdin); assert "connections" in obj, obj; print("ok")'

if [ "$SKIP_WS" = "1" ]; then
  say "WS skipped (SKIP_WS=1)"
  exit 0
fi

say "WS /ws"
BASE_URL="$BASE_URL" \
WS_URL="$WS_URL" \
JWT_ALG="$JWT_ALG" \
JWT_USER_ID="$JWT_USER_ID" \
JWT_PRIVATE_KEY_FILE="$JWT_PRIVATE_KEY_FILE" \
python scripts/ws_smoke.py

if [ "$SKIP_PUBLISH" = "1" ]; then
  say "WS publish skipped (SKIP_PUBLISH=1)"
  exit 0
fi

say "WS publish/receive"
BASE_URL="$BASE_URL" \
WS_URL="$WS_URL" \
JWT_ALG="$JWT_ALG" \
JWT_USER_ID="$JWT_USER_ID" \
JWT_PRIVATE_KEY_FILE="$JWT_PRIVATE_KEY_FILE" \
GATEWAY_API_KEY="$GATEWAY_API_KEY" \
python scripts/ws_publish_smoke.py

say "done"
