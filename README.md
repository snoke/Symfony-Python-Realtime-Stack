# Symfony + Python Realtime Stack — Branch Overview

Dieses `main`‑Branch ist bewusst minimal. Wähle eine Architektur und checke den passenden Branch aus:

## `terminator` (Symfony‑first)
- WebSocket‑Gateway + **Webhook/HTTP‑Presence**
- Schnell in bestehende Symfony‑Apps integrierbar
- Guter Fit für klassische Apps mit moderatem Realtime‑Anteil
- **Warum nicht nur Mercure?** Mercure = SSE, nicht bidirektionales WS (kein echter Client→Server‑Kanal)

## `realtime-core` (Broker‑first)
- **Kein Webhook**, Events gehen **nur** über Broker (Redis/RabbitMQ)
- Gateway bleibt nahezu stateless, Presence in Redis
- Skalierbar für hohe Connection‑Zahlen (kein Symfony‑Boot pro Message)
- Symfony ist Producer/Consumer, nicht WS‑Terminator

## Datenhoheit / DSGVO (Self‑Hosted)
- Verbindungen, Presence und Events liegen in **deiner** Infrastruktur
- Retention/TTL steuerst du selbst (Redis/Broker)
- DSGVO‑Pflichten (Löschung, Auskunft, Zweckbindung) bleiben bei dir – aber sind technisch erfüllbar

## Start
- `git checkout terminator`
- `git checkout realtime-core`

Die jeweiligen Branches enthalten eine eigene README mit Setup und Demo‑Schritten.
