import asyncio
import json
import os
import sys
import time

try:
    import jwt
    import websockets
except Exception as exc:
    print("missing dependency:", exc, file=sys.stderr)
    print("install: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(2)

WS_URL = os.getenv("WS_URL", "ws://localhost:8180/ws")
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


async def main() -> None:
    token = make_token()
    headers = {"Authorization": f"Bearer {token}"}
    async with websockets.connect(WS_URL, extra_headers=headers) as ws:
        await ws.send(json.dumps({"type": "ping"}))
        msg = await asyncio.wait_for(ws.recv(), timeout=TIMEOUT_SECONDS)
        print("received:", msg)


if __name__ == "__main__":
    asyncio.run(main())
