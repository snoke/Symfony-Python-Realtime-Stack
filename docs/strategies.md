# Strategies

This document captures the optional, configurable strategy layers for the stack.

## Ordering Strategy (core)
You can choose how ordering keys are derived for brokered events:
- `topic` strategy uses a message field (default `topic`, fallback to `type`).
- `subject` strategy uses the connection subject (default `user:{id}`) or explicit message `subject`.
- `none` disables ordering keys.

Partitioning (optional):
- If `ORDERING_PARTITION_MODE=suffix`, the gateway appends the ordering key to the broker stream/routing key.
- Example: `ws.inbox.room:123` (Redis stream or RabbitMQ routing key).

## Observability / Tracing Strategy
Status: OTel end-to-end tracing implemented.

Goal: real spans + propagation across Gateway → Broker → Symfony.
Includes producer/consumer spans for broker publish + outbox delivery.

Strategies:
- `none`: no tracing
- `propagate`: forward trace headers if present
- `full`: always create spans and propagate

Gateway policies:
- `TRACING_TRACE_ID_FIELD`
- `TRACING_HEADER_NAME`
- `TRACING_SAMPLE_RATE`
- `TRACING_EXPORTER=stdout|otlp|none`
- `OTEL_SERVICE_NAME`
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `OTEL_EXPORTER_OTLP_PROTOCOL`

Symfony policies:
- `WS_TRACING_ENABLED=0|1`
- `WS_TRACING_EXPORTER=stdout|otlp|none`
- `WS_TRACEPARENT_FIELD`
- `WS_TRACE_ID_FIELD`
- `OTEL_SERVICE_NAME`
- `OTEL_EXPORTER_OTLP_ENDPOINT`
- `OTEL_EXPORTER_OTLP_PROTOCOL`

### Tracing Smoke Checks (real)
1. Start the core stack (with realtime core enabled).
2. Runtime deps check (gateway + symfony):
   ```
   COMPOSE_FILES="docker-compose.yaml docker-compose.realtime-core.yaml" \
     ./scripts/tracing_runtime_check.sh
   ```
3. End-to-end propagation check (traceparent + trace_id):
   ```
   ./scripts/tracing_e2e_check.sh
   ```

## Replay / Persistence Strategy
Status: implemented for Redis streams + RabbitMQ minimal + replay API hardening.

Goal: define replay behavior for brokered events without hard-coding retention.

Strategies:
- `none`: no replay guarantees
- `bounded`: replay within a bounded window (stream maxlen / TTL)
- `durable`: external persistence for long-term replay

Policies:
- `REPLAY_RETENTION_SECONDS`
- `REPLAY_MAXLEN`
- `REPLAY_SNAPSHOT_SECONDS`

RabbitMQ minimal replay (durable + TTL/DLX):
- `RABBITMQ_QUEUE_TTL_MS`
- `RABBITMQ_INBOX_QUEUE`
- `RABBITMQ_INBOX_QUEUE_TTL_MS`
- `RABBITMQ_EVENTS_QUEUE`
- `RABBITMQ_EVENTS_QUEUE_TTL_MS`

RabbitMQ robust replay (API):
- `POST /internal/replay/rabbitmq` with `X-API-Key` header (or `{ api_key, limit }` payload)
- `RABBITMQ_REPLAY_TARGET_EXCHANGE`
- `RABBITMQ_REPLAY_TARGET_ROUTING_KEY`
- `RABBITMQ_REPLAY_MAX_BATCH`

Replay API hardening (defaults, configurable):
- API key required: `REPLAY_API_KEY` (fallback to `GATEWAY_API_KEY`)
- Rate limiting: `REPLAY_RATE_LIMIT_STRATEGY=in_memory|redis|none`
- Idempotency (optional): `REPLAY_IDEMPOTENCY_STRATEGY=none|in_memory|redis`
- Audit logging: `REPLAY_AUDIT_LOG=1`

Example request:
```
curl -X POST http://localhost:8180/internal/replay/rabbitmq \
  -H "X-API-Key: $REPLAY_API_KEY" \
  -H "Idempotency-Key: replay-2025-01-01" \
  -H "Content-Type: application/json" \
  -d '{"limit":100}'
```

## Presence Strategy
Status: gateway + Symfony interpretation + Symfony writer.

Goal: make presence consistency configurable via strategy pattern and allow Symfony-specific interpretation.

Strategies:
- `ttl`: rely on Redis TTL + periodic refresh
- `heartbeat`: client heartbeats drive presence updates
- `session`: explicit connect/disconnect lifecycle only

Gateway policies:
- `PRESENCE_TTL_SECONDS`
- `PRESENCE_HEARTBEAT_SECONDS`
- `PRESENCE_GRACE_SECONDS`
- `PRESENCE_REFRESH_ON_MESSAGE`

Symfony interpretation policies:
- `WS_PRESENCE_INTERPRETATION_STRATEGY=none|ttl|heartbeat|session`
- `WS_PRESENCE_TTL_SECONDS`
- `WS_PRESENCE_HEARTBEAT_SECONDS`
- `WS_PRESENCE_GRACE_SECONDS`
- `WS_PRESENCE_USE_LAST_SEEN`

Symfony writer (ownership) policies:
- `WS_PRESENCE_WRITER_TYPE=none|redis`
- `WS_PRESENCE_WRITER_TTL_SECONDS`
- `WS_PRESENCE_WRITER_REFRESH_ON_MESSAGE`

Note: if Symfony owns presence, disable gateway presence updates to avoid double-writes.

## Backpressure Strategy
Status: implemented in gateway.

Goal: make backpressure policies configurable and swappable like `WS_MODE`.

Strategies:
- `none`: no backpressure logic (legacy behavior)
- `drop`: drop messages when limits are hit
- `close`: close connections under sustained pressure
- `buffer`: bounded buffering (per-connection + global)

Policies:
- `BACKPRESSURE_PER_CONN_BUFFER`
- `BACKPRESSURE_GLOBAL_BUFFER`
- `BACKPRESSURE_MAX_INFLIGHT`
- `BACKPRESSURE_DROP_POLICY=oldest|newest`
