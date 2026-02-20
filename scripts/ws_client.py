import asyncio
import json
import os
import time
import jwt
import websockets

WS_URL = os.getenv("WS_URL", "ws://localhost/ws")
JWT_SECRET = os.getenv("JWT_SECRET", "dev-secret")
JWT_ALG = os.getenv("JWT_ALG", "HS256")
JWT_USER_ID = os.getenv("JWT_USER_ID", "42")


def make_token() -> str:
    payload = {
        "user_id": JWT_USER_ID,
        "iat": int(time.time()),
        "exp": int(time.time()) + 3600,
    }
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALG)


async def main() -> None:
    token = make_token()
    headers = {"Authorization": f"Bearer {token}"}
    async with websockets.connect(WS_URL, extra_headers=headers) as ws:
        await ws.send(json.dumps({"type": "ping"}))
        msg = await ws.recv()
        print("received:", msg)


if __name__ == "__main__":
    asyncio.run(main())
