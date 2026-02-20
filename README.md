# Symfony + Python WebSocket Bundle Stack

This repo contains:
- `gateway/`: Python WebSocket gateway (FastAPI)
- `symfony/`: Symfony app + reusable bundle
- `traefik/`: Reverse proxy config

Default behavior:
- WebSocket connections terminate at the Python gateway.
- Symfony publishes push events to the gateway via HTTP.
- Presence is read from the gateway via HTTP.
- Events are delivered to Symfony via a webhook (enabled by default).

## Quick start (dev)
1. Build and run:
   - `docker compose -f docker-compose.yaml -f docker-compose.local.yaml up --build`
2. Open:
   - WebSocket: `ws://localhost:8180/ws`
   - Symfony: `http://localhost:8180/api/ping`

## Minimal WS test client
This uses HS256 for local dev only.

1. Install deps:
   - `python3 -m venv .venv && source .venv/bin/activate`
   - `pip install -r scripts/requirements.txt`
2. Run:
   - `JWT_SECRET=dev-secret WS_URL=ws://localhost:8180/ws python scripts/ws_client.py`

You should see `received: {"type":"pong"}`.

## Notes
- This is a scaffold. For production, add Redis/RabbitMQ, persistence, and rate limits.
- For production, configure RS256 (JWKS or public key) in `gateway`.
