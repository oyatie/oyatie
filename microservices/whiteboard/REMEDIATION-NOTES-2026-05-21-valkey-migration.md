## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-adapter-valkey.yaml
- microservices/whiteboard/performance-benchmark-numbers-2026-05-20.md
- microservices/whiteboard/IP-014-marketplace-dealset-settlement.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary or a grep false positive.

Files renamed (git mv):
- microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-adapter-redis.yaml -> microservices/whiteboard/catalog/oya-whiteboard-canvas-collaboration-adapter-valkey.yaml
