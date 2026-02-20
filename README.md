# Symfony + Python Realtime Stack — Branch Overview

This `main` branch is intentionally minimal. Pick an architecture and check out the matching branch.

## `terminator` (Symfony‑first)
- WebSocket gateway + **webhook/HTTP presence**
- Fast to integrate into existing Symfony apps
- Good fit for classic apps with moderate realtime
- **Why not just Mercure?** Mercure = SSE, not bidirectional WS (no true client→server channel)

## `realtime-core` (broker‑first)
- **No webhook**, events go **only** through the broker (Redis/RabbitMQ)
- Gateway is mostly stateless, presence in Redis
- Scales to high connection counts (no Symfony boot per message)
- Symfony is producer/consumer, not WS terminator

## Data sovereignty / GDPR (self‑hosted)
- Connections, presence and events stay in **your** infrastructure
- Retention/TTL is yours to control (Redis/Broker)
- GDPR duties (erasure, access, purpose limitation) remain with you — but are technically enforceable

## Start
- `git checkout terminator`
- `git checkout realtime-core`

Each branch contains its own README with setup and demo steps.
