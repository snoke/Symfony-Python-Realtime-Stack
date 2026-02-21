# Ops Notes

## RabbitMQ Policies (optional, infra-level)
```
rabbitmqctl set_policy ws-replay-ttl "^ws\.(inbox|events)$" '{"message-ttl":600000,"max-length":100000,"dead-letter-exchange":"ws.dlq"}' --apply-to queues
rabbitmqctl set_policy ws-replay-lazy "^ws\.(inbox|events)$" '{"queue-mode":"lazy"}' --apply-to queues
```

## Monitoring / Alarms (suggested)
- Track DLQ depth + inbox/event queue depth.
- Alert on spikes in `rabbitmq_replay_total` or repeated replays.
- Observe replay API rate limits via `/metrics` counters.

## Push Demo (optional)
The chat demo is the primary UI demo. For headless checks you can still use:
```
./scripts/push_demo.sh
```
Core mode:
```
WS_MODE=core ./scripts/push_demo.sh
```
