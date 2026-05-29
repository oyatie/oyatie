## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/docs/ARCHITECTURE.md
- microservices/docs/AUDIT-FINDINGS-2026-05-18.json
- microservices/docs/IP-001-iac-bootstrap.md
- microservices/docs/IP-007-collab-crdt-adapter-valkey-worker.md
- microservices/docs/PHASE-01-DOCS-FOUNDATION.md
- microservices/docs/PRD.md
- microservices/docs/REMEDIATION-NOTES-2026-05-21-tier-scrub.md
- microservices/docs/catalog/oya-docs-collab-crdt-adapter-valkey.yaml
- microservices/docs/coherence-audit-2026-05-20.md
- microservices/docs/compliance.md
- microservices/docs/decisions/ADR-DOCS-0003-export-pipeline-architecture.md
- microservices/docs/iac/helm/Chart.yaml
- microservices/docs/iac/helm/templates/networkpolicy.yaml
- microservices/docs/iac/helm/values.yaml
- microservices/docs/manifest.json
- microservices/docs/migration-from-connect.md
- microservices/docs/runbooks/collab-conflict-resolution.md
- microservices/docs/threat-model.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary or a grep false positive.

Files renamed (git mv):
- microservices/docs/IP-007-collab-crdt-adapter-redis-worker.md -> microservices/docs/IP-007-collab-crdt-adapter-valkey-worker.md
- microservices/docs/catalog/oya-docs-collab-crdt-adapter-redis.yaml -> microservices/docs/catalog/oya-docs-collab-crdt-adapter-valkey.yaml
