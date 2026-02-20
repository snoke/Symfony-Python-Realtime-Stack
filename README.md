# Symfony + Python WebSocket Stack (Configurable)

This repo provides a Python WebSocket gateway + a Symfony bundle.  
Choose the architecture **at runtime** via `WS_MODE`.

## Modes (WS_MODE)
**terminator (default)**  
- Gateway terminates WS + JWT and **webhooks Symfony**.  
- Presence via HTTP (`/internal/connections`).  
- Symfony stays fully in control (classic app integration).

**core**  
- **Broker-first**: gateway publishes events to Redis/RabbitMQ.  
- Presence lives in Redis.  
- Symfony is producer/consumer (no Symfony boot per message).

### Events routing (gateway → Symfony)
`EVENTS_MODE=webhook|broker|both|none`  
Defaults to `webhook` in `terminator`, `broker` in `core`.

---

## Quick start (terminator)
1. Generate dev keys (RS256):  
   - `./scripts/gen_dev_keys.sh`
2. Build + run:  
   - `docker compose -f docker-compose.yaml -f docker-compose.local.yaml up --build`
3. Open:  
   - WebSocket: `ws://localhost:8180/ws`  
   - API: `http://localhost:8180/api/ping`

Gateway → Symfony webhook is enabled by default:  
`SYMFONY_WEBHOOK_URL=http://symfony:8000/internal/ws/events`

## Quick start (core)
1. Generate dev keys (RS256):  
   - `./scripts/gen_dev_keys.sh`
2. Build + run (broker-first):  
   - `docker compose -f docker-compose.yaml -f docker-compose.local.yaml -f docker-compose.realtime-core.yaml up --build`
3. Open:  
   - WebSocket: `ws://localhost:8180/ws`  
   - API: `http://localhost:8180/api/ping`

---

## Minimal WS test client
This uses RS256 for local dev.

1. Install deps:
   - `python3 -m venv .venv && source .venv/bin/activate`
   - `pip install -r scripts/requirements.txt`
2. Run:
   - `JWT_PRIVATE_KEY_FILE=./scripts/keys/dev_private.pem WS_URL=ws://localhost:8180/ws python scripts/ws_client.py`

To send a demo message on connect:
- `WS_SEND_MESSAGE=1 WS_MESSAGE_JSON='{"type":"chat","payload":"hello world"}' JWT_PRIVATE_KEY_FILE=./scripts/keys/dev_private.pem WS_URL=ws://localhost:8180/ws python scripts/ws_client.py`

You should see `received: {"type":"pong"}`.

## Push demo (terminator)
1. Start the WS client in one terminal.
2. Trigger a push from Symfony:
   - `./scripts/push_demo.sh`

---

## Event schema (gateway → webhook/broker)
Event types:
- `connected`
- `disconnected`
- `message_received`

Common fields:
```
{
  "type": "connected|disconnected|message_received",
  "connection_id": "uuid",
  "user_id": "42",
  "subjects": ["user:42"],
  "connected_at": 1700000000
}
```

`message_received` extra fields:
```
{
  "message": { "type": "chat", "payload": "hello world" },
  "raw": "{\"type\":\"chat\",\"payload\":\"hello world\"}"
}
```

Edge cases:
- Invalid JWT → WS closed with `4401`.
- `ping` messages are answered with `pong` and **not** published.
- Non-JSON WS messages become `{"type":"raw","payload":"<text>"}`.
- Rate-limited clients receive `{"type":"rate_limited"}`.

---

## Symfony config (overview)
Mode + transport/presence/events are configurable in `symfony/config/packages/snoke_ws.yaml`.

Key env vars:
- `WS_MODE=terminator|core`
- `EVENTS_MODE=webhook|broker|both|none`
- `SYMFONY_WEBHOOK_URL` + `SYMFONY_WEBHOOK_SECRET` (terminator)
- `WS_GATEWAY_BASE_URL` + `WS_GATEWAY_API_KEY` (Symfony → gateway)
- `WS_REDIS_DSN`, `WS_RABBITMQ_DSN`, … (core/broker)

---

## Quick start (prod compose)
1. Set env:
   - `cp .env.example .env` and edit
2. Create ACME storage:
   - `touch traefik/acme.json && chmod 600 traefik/acme.json`
3. Run:
   - `docker compose -f docker-compose.yaml -f docker-compose.prod.yaml up -d --build`

---

## Security controls
Gateway JWT validation:
- `JWT_ISSUER` (optional)
- `JWT_AUDIENCE` (optional)
- `JWT_LEEWAY` (seconds, optional)

---

## Data sovereignty / GDPR (self-hosted)
- Connections, presence and events stay in **your** infrastructure.
- Retention is controlled via Redis TTL / broker retention.
- GDPR duties (erasure, access, purpose limitation) remain with you.

---

## Branch snapshots (optional)
If you want the old split snapshots:
- `git checkout terminator`
- `git checkout realtime-core`

---

## Brokers (Redis/RabbitMQ)
RabbitMQ Management UI:
- `http://localhost:8167` (user/pass: `guest` / `guest`)
