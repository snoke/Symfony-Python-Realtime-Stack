import asyncio
import json
import os
import sys
import time
import urllib.request

try:
    import jwt
    import websockets
except Exception as exc:
    print("missing dependency:", exc, file=sys.stderr)
    print("install: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(2)

BASE_URL = os.getenv("BASE_URL", "http://localhost:8180")
WS_URL = os.getenv("WS_URL", "ws://localhost:8180/ws")
GATEWAY_API_KEY = os.getenv("GATEWAY_API_KEY", "dev-key")
JWT_SECRET = os.getenv("JWT_SECRET", "dev-secret")
JWT_ALG = os.getenv("JWT_ALG", "RS256")
JWT_USER_ID = os.getenv("JWT_USER_ID", "42")
JWT_PRIVATE_KEY_FILE = os.getenv("JWT_PRIVATE_KEY_FILE", "")
TIMEOUT_SECONDS = float(os.getenv("WS_TIMEOUT", "5"))


def make_token() -> str:
    payload = {
        "user_id": JWT_USER_ID,
        "iat": int(time.time()),
        "exp": int(time.time()) + 3600,
    }
    if JWT_ALG.upper().startswith("RS"):
        if not JWT_PRIVATE_KEY_FILE:
            raise RuntimeError("JWT_PRIVATE_KEY_FILE is required for RS256")
        with open(JWT_PRIVATE_KEY_FILE, "r", encoding="utf-8") as f:
            private_key = f.read()
        return jwt.encode(payload, private_key, algorithm=JWT_ALG)
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALG)


def publish_http(subject: str, payload: dict) -> None:
    body = json.dumps(
        {
            "api_key": GATEWAY_API_KEY,
            "subjects": [subject],
            "payload": payload,
        }
    ).encode("utf-8")
    req = urllib.request.Request(
        f"{BASE_URL}/internal/publish",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=5) as resp:
        resp.read()


async def main() -> None:
    token = make_token()
    headers = {"Authorization": f"Bearer {token}"}
    subject = f"user:{JWT_USER_ID}"
    expected = {"type": "smoke", "payload": "hello"}

    async with websockets.connect(WS_URL, extra_headers=headers) as ws:
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(None, publish_http, subject, expected)

        deadline = loop.time() + TIMEOUT_SECONDS
        while True:
            remaining = deadline - loop.time()
            if remaining <= 0:
                raise TimeoutError("timed out waiting for event")
            msg = await asyncio.wait_for(ws.recv(), timeout=remaining)
            print("received:", msg)
            data = json.loads(msg)
            if data.get("type") == "event" and data.get("payload") == expected:
                return


if __name__ == "__main__":
    asyncio.run(main())
