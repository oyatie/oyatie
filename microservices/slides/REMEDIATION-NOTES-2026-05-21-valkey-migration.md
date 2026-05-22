## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/slides/AUDIT-FINDINGS-2026-05-18.json
- microservices/slides/IP-001-layer-a-cdn-postgres-valkey-s3-ws-gateway-iac.md
- microservices/slides/IP-005-real-time-collaboration-loro-kernel-domain-adapter.md
- microservices/slides/IP-006-real-time-collaboration-worker-sdk.md
- microservices/slides/PHASE-01-SLIDES-FOUNDATION.md
- microservices/slides/PRD.md
- microservices/slides/catalog/oya-slides-real-time-collaboration-adapter-valkey.yaml
- microservices/slides/coherence-audit-2026-05-20.md
- microservices/slides/decisions/ADR-SLIDES-0001-crdt-library-selection.md
- microservices/slides/failure-modes.md
- microservices/slides/iac/helm/templates/deployment.yaml
- microservices/slides/iac/helm/values.yaml
- microservices/slides/manifest.json
- microservices/slides/performance-benchmark-numbers-2026-05-20.md
- microservices/slides/runbooks/collab-conflict-resolution-crdt.md
- microservices/slides/threat-model.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary.

Files renamed (git mv):
- microservices/slides/IP-001-layer-a-cdn-postgres-redis-s3-ws-gateway-iac.md -> microservices/slides/IP-001-layer-a-cdn-postgres-valkey-s3-ws-gateway-iac.md
- microservices/slides/catalog/oya-slides-real-time-collaboration-adapter-redis.yaml -> microservices/slides/catalog/oya-slides-real-time-collaboration-adapter-valkey.yaml
