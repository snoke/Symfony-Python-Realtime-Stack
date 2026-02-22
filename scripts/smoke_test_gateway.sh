#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:8180}"
WS_URL="${WS_URL:-ws://localhost:8180/ws}"
JWT_PRIVATE_KEY_FILE="${JWT_PRIVATE_KEY_FILE:-./scripts/keys/dev_private.pem}"
JWT_ALG="${JWT_ALG:-RS256}"
JWT_USER_ID="${JWT_USER_ID:-4242}"

say() {
  printf "[smoke] %s\n" "$*"
}

say "HTTP /health"
curl -fsS "$BASE_URL/health" >/dev/null

say "HTTP /ready"
curl -fsS "$BASE_URL/ready" >/dev/null

say "HTTP /metrics"
curl -fsS "$BASE_URL/metrics" | grep -q "^ws_connections_total "

say "HTTP /internal/connections"
curl -fsS "$BASE_URL/internal/connections" >/dev/null

if [ ! -d .venv ]; then
  python3 -m venv .venv
fi
# shellcheck disable=SC1091
source .venv/bin/activate
pip install -r scripts/requirements.txt >/dev/null

say "WS ping/pong + outbox delivery"
python - <<'PY'
import asyncio
import json
import os
import subprocess
import time

import jwt
import websockets

BASE_URL = os.getenv("BASE_URL", "http://localhost:8180")
WS_URL = os.getenv("WS_URL", "ws://localhost:8180/ws")
JWT_ALG = os.getenv("JWT_ALG", "RS256")
JWT_USER_ID = os.getenv("JWT_USER_ID", "4242")
JWT_PRIVATE_KEY_FILE = os.getenv("JWT_PRIVATE_KEY_FILE", "./scripts/keys/dev_private.pem")

payload = {
    "user_id": JWT_USER_ID,
    "iat": int(time.time()),
    "exp": int(time.time()) + 3600,
}
if JWT_ALG.upper().startswith("RS"):
    if not JWT_PRIVATE_KEY_FILE:
        raise SystemExit("JWT_PRIVATE_KEY_FILE is required for RS256")
    with open(JWT_PRIVATE_KEY_FILE, "r", encoding="utf-8") as f:
        private_key = f.read()
    token = jwt.encode(payload, private_key, algorithm=JWT_ALG)
else:
    token = jwt.encode(payload, "dev-secret", algorithm=JWT_ALG)


def build_compose_cmd() -> list[str]:
    compose_files = os.getenv("COMPOSE_FILES", "").strip().split()
    if not compose_files:
        if os.path.exists("docker-compose.realtime-core.yaml"):
            compose_files = ["docker-compose.yaml", "docker-compose.realtime-core.yaml"]
        else:
            compose_files = ["docker-compose.yaml"]
    cmd = ["docker", "compose"]
    for fname in compose_files:
        cmd += ["-f", fname]
    return cmd


async def main() -> None:
    headers = {"Authorization": f"Bearer {token}"}
    async with websockets.connect(WS_URL, extra_headers=headers) as ws:
        await ws.send(json.dumps({"type": "ping"}))
        pong = await asyncio.wait_for(ws.recv(), timeout=3)
        if '"type":"pong"' not in pong:
            raise SystemExit(f"unexpected pong payload: {pong}")

        outbox_payload = {
            "subjects": [f"user:{JWT_USER_ID}"],
            "payload": {"type": "chat", "text": "smoke-outbox", "ts": int(time.time())},
        }
        outbox_body = json.dumps(outbox_payload, separators=(",", ":"), sort_keys=True)
        cmd = build_compose_cmd() + [
            "exec",
            "-T",
            "redis",
            "redis-cli",
            "XADD",
            "ws.outbox",
            "*",
            "data",
            outbox_body,
        ]
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise SystemExit(f"redis XADD failed: {result.stderr.strip()}")

        deadline = time.time() + 5
        while time.time() < deadline:
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=1)
            except asyncio.TimeoutError:
                continue
            try:
                data = json.loads(msg)
            except Exception:
                continue
            if data.get("type") == "event":
                payload = data.get("payload") or {}
                if payload.get("type") == "chat" and payload.get("text") == "smoke-outbox":
                    print("received:", msg)
                    return
        raise SystemExit("timeout waiting for outbox delivery")


asyncio.run(main())
PY

say "done"
