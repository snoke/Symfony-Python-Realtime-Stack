## Core stack details
What you get in `core` mode:
- Stateless gateway publishes events to broker(s)
- Symfony acts as producer/consumer (no webhook round‑trip)
- `symfony-consumer` service reads `ws.inbox` and updates `/api/ws/last-message`

Core flow:
1. Client → Gateway (WS message)
2. Gateway → Broker (`ws.inbox` stream / queue)
3. Symfony consumer → reads event → app logic

Optional: run consumer manually (if you don't use the service):
```
docker compose -f docker-compose.yaml -f docker-compose.realtime-core.yaml exec -T symfony php bin/console ws:consume
```

Useful env vars in core:
- `WS_MODE=core`
- `EVENTS_MODE=broker|webhook|both|none`
- `WS_REDIS_DSN` / `WS_RABBITMQ_DSN`
- `WS_CONSUMER_LOG_LEVEL`

Verify core wiring quickly:
1. Start the WS client.
2. Send a demo message.
3. Check `/api/ws/last-message` (updated by the consumer).

## Terminator stack details
What you get in `terminator` mode:
- Gateway calls Symfony via webhook (`/internal/ws/events`)
- Symfony publishes via HTTP to the gateway
- Presence can be resolved via HTTP (no brokers required)

Useful env vars in terminator:
- `WS_MODE=terminator`
- `WS_GATEWAY_BASE_URL` / `WS_GATEWAY_API_KEY`
- `SYMFONY_WEBHOOK_URL` / `SYMFONY_WEBHOOK_SECRET`

Demo mapping (core): `message_received` → `chat` is handled by `ChatDemoListener`
(publisher uses subjects like `user:{id}`).


## Minimal WS Test Client
1. Install dependencies:
   ```
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r scripts/requirements.txt
   ```
2. Run client:
   ```
   JWT_PRIVATE_KEY_FILE=./scripts/keys/dev_private.pem WS_URL=ws://localhost:8180/ws python scripts/ws_client.py
   ```
3. Send a demo message on connect:
   ```
   WS_SEND_MESSAGE=1 WS_MESSAGE_JSON='{"type":"chat","payload":"hello world"}' \
   JWT_PRIVATE_KEY_FILE=./scripts/keys/dev_private.pem WS_URL=ws://localhost:8180/ws \
   python scripts/ws_client.py
   ```

Expected response: `{"type":"pong"}`


## Event Schema (gateway → webhook/broker)
Event types: `connected`, `disconnected`, `message_received`

Common fields:
```
type: connected|disconnected|message_received
connection_id: uuid
user_id: 42
subjects: ["user:42"]
connected_at: 1700000000
```

`message_received` extra fields:
```
message: { type: chat, payload: hello world }
raw: {"type":"chat","payload":"hello world"}
traceparent: 00-... (optional, W3C)
ordering_key: room:123 (optional)
ordering_strategy: topic|subject (optional)
```

Edge cases:
- Invalid JWT → WS closed with `4401`
- `ping` messages → `pong` (not published)
- Non-JSON WS messages → `{ "type":"raw","payload":"" }`
- Rate-limited clients → `{ "type":"rate_limited" }`


## Symfony Config Overview
Mode + transport/presence/events configurable in `symfony/config/packages/snoke_ws.yaml`

Minimal example (make sure `subjects` is set):
```
snoke_ws:
  subjects:
    user_prefix: "user:"
```

Token helper (service, opt-in):
```
use Snoke\WsBundle\Service\DemoTokenService;

public function token(DemoTokenService $tokens): Response
{
    [$jwt, $error] = $tokens->issue('42');
    // return Response/JsonResponse with $jwt or $error
}
```

Key env vars:
- `WS_MODE=terminator|core`
- `EVENTS_MODE=webhook|broker|both|none`
- `LOG_LEVEL`, `LOG_FORMAT` (gateway)
- `WS_CONSUMER_LOG_LEVEL` (core consumer)
- `SYMFONY_WEBHOOK_URL` + `SYMFONY_WEBHOOK_SECRET` (terminator)
- `WS_GATEWAY_BASE_URL` + `WS_GATEWAY_API_KEY` (Symfony → gateway)
- `WS_REDIS_DSN`, `WS_RABBITMQ_DSN`, … (core/broker)
- `ORDERING_STRATEGY=none|topic|subject` (gateway)
- `ORDERING_TOPIC_FIELD` (gateway, `topic`)
- `ORDERING_SUBJECT_SOURCE=user|subject` (gateway)
- `ORDERING_PARTITION_MODE=none|suffix` (gateway)
- `ORDERING_PARTITION_MAX_LEN` (gateway)


## Production Quickstart
1. Set env: `cp .env.example .env`
2. Create ACME storage:
   ```
   touch traefik/acme.json && chmod 600 traefik/acme.json
   ```
3. Run:
   ```
   docker compose -f docker-compose.yaml -f docker-compose.prod.yaml up -d --build
   ```


## Healthchecks
- Gateway: `GET /health`
- Symfony: `GET /api/ping`


## Brokers (Redis/RabbitMQ)
RabbitMQ Management UI: `http://localhost:8167` (user/pass: `guest` / `guest`)


## More Docs
- Strategy details: `docs/strategies.md`
- Ops notes: `docs/ops.md`
